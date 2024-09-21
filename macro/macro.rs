#[proc_macro_attribute]
pub fn jni(
	attrs: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	generate_jni_wrapper(
		syn::parse_macro_input!(attrs),
		syn::parse_macro_input!(input),
	)
		.unwrap()
		.into()
}




use proc_macro2::{Span, TokenStream, TokenTree};
use quote::TokenStreamExt;
use syn::{FnArg, Item, ReturnType, Type};

fn generate_jni_wrapper(attrs: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
	let mut out = TokenStream::new();

	let Item::Fn(fn_item) = syn::parse2(input.clone())? else {
		return Err(syn::Error::new(Span::call_site(), "#[jni] is only supported on functions"));
	};

	let mut what_next = WhatNext::Nothing;

	let mut package = None;
	let mut clazz = None;
	let mut exception = None;
	let mut return_pointer = false;

	for attr in attrs {
		match what_next {
			WhatNext::Nothing => {
				if let TokenTree::Ident(ref i) = attr {
					match i.to_string().as_ref() {
						"package" => what_next = WhatNext::Package,
						"class" => what_next = WhatNext::Class,
						"exception" => what_next = WhatNext::Exception,
						"ptr" => return_pointer = true,
						_ => return Err(syn::Error::new(Span::call_site(), "unexpected attribute on macro: {attr}")),
					}
				}
			},
			WhatNext::Class => {
				if let TokenTree::Literal(i) = attr {
					let raw = i.to_string().replace('"', "");
					clazz = Some(syn::Ident::new(&raw, Span::call_site()));
					what_next = WhatNext::Nothing;
				}
			},
			WhatNext::Package => {
				if let TokenTree::Literal(i) = attr {
					let raw = i.to_string().replace('"', "").replace(".", "_");
					package = Some(syn::Ident::new(&raw, Span::call_site()));
					what_next = WhatNext::Nothing;
				}
			},
			WhatNext::Exception => {
				if let TokenTree::Literal(i) = attr {
					let raw = i.to_string().replace('"', "").replace(".", "_");
					exception = Some(raw);
					what_next = WhatNext::Nothing;
				}
			}
		}
	}

	let Some(package) = package else { return Err(syn::Error::new(Span::call_site(), "missing attribute 'package'")) };
	let Some(clazz) = clazz else { return Err(syn::Error::new(Span::call_site(), "missing attribute 'class'")) };

	let (could_error, ret_type) = match fn_item.sig.output {
		syn::ReturnType::Default => (false, fn_item.sig.output),
		syn::ReturnType::Type(_tok, ty) => match *ty {
			syn::Type::Path(ref path) => {
				let Some(last) = path.path.segments.last() else {
					return Err(syn::Error::new(Span::call_site(), "empty Result type is not valid"));
				};

				// TODO this is terrible, macro returns a function and we call it?? there must be a
				// better way!!!
				let mut out = (
					false,
					ReturnType::Type(syn::Token![->](Span::call_site()), Box::new(Type::Path(path.clone())))
				);

				if last.ident == "Result" {
					match &last.arguments {
						syn::PathArguments::None => return Err(syn::Error::new(Span::call_site(), "Result without generics is not valid")),
						syn::PathArguments::Parenthesized(_) => return Err(syn::Error::new(Span::call_site(), "Parenthesized Result is not valid")),
						syn::PathArguments::AngleBracketed(ref generics) => for generic in generics.args.iter() {
							match generic {
								syn::GenericArgument::Lifetime(_) => continue,
								syn::GenericArgument::Type(ty) => {
									out = (true, ReturnType::Type(syn::Token![->](Span::call_site()), Box::new(ty.clone())));
									break;
								},
								_ => return Err(syn::Error::new(Span::call_site(), "unexpected type in Result")),
							}
						}
					}
				}

				out
			},
			_ => return Err(syn::Error::new(Span::call_site(), "unsupported return type")),
		},
	};


	let mut incoming = TokenStream::new();
	let mut forwarding = TokenStream::new();

	for arg in fn_item.sig.inputs {
		let FnArg::Typed(ty) = arg else {
			return Err(syn::Error::new(Span::call_site(), "#[jni] macro doesn't work on methods"));
		};
		incoming.append_all(quote::quote!( #ty , ));
		let pat = unpack_pat(*ty.pat)?;
		forwarding.append_all(pat);
	}

	let name = fn_item.sig.ident.to_string();
	let name_jni = name.replace("_", "_1");
	let fn_name_inner = syn::Ident::new(&name, Span::call_site());
	let fn_name = syn::Ident::new(&format!("Java_{package}_{clazz}_{name_jni}"), Span::call_site());

	let Some(env_ident) = forwarding.clone().into_iter().next() else {
		return Err(syn::Error::new(Span::call_site(), "missing JNIEnv argument"));
	};

	let return_expr = if return_pointer {
		quote::quote!( std::ptr::null_mut() )
	} else {
		quote::quote!( 0 )
	};

	let wrapped = if could_error {
		if let Some(exception) = exception {
			// V----------------------------------V
			quote::quote! {
				#[no_mangle]
				#[allow(unused_mut)]
				pub extern "system" fn #fn_name<'local>(#incoming) #ret_type {
					use jni_toolbox::JniToolboxError;
					match #fn_name_inner(#forwarding) {
						Ok(ret) => ret,
						Err(e) => match #env_ident.throw_new(#exception, format!("{e:?}")) {
							Ok(_) => return #return_expr,
							Err(e) => panic!("error throwing java exception: {e}"),
						}
					}
				}
			}
			// ^----------------------------------^
		} else {
			// V----------------------------------V
			quote::quote! {
				#[no_mangle]
				#[allow(unused_mut)]
				pub extern "system" fn #fn_name<'local>(#incoming) #ret_type {
					use jni_toolbox::JniToolboxError;
					// NOTE: this is SAFE! the cloned env reference lives less than the actual one, we just lack a
					//       way to get it back from the called function and thus resort to unsafe cloning
					let mut env_copy = unsafe { #env_ident.unsafe_clone() };
					match #fn_name_inner(#forwarding) {
						Err(e) => match env_copy.find_class(e.jclass()) {
							Err(e) => panic!("error throwing Java exception -- failed resolving error class: {e}"),
							Ok(class) => match env_copy.new_string(format!("{e:?}")) {
								Err(e) => panic!("error throwing Java exception --  failed creating error string: {e}"),
								Ok(msg) => match env_copy.new_object(class, "(Ljava/lang/String;)V", &[jni::objects::JValueGen::Object(&msg)]) {
									Err(e) => panic!("error throwing Java exception -- failed creating object: {e}"),
									Ok(obj) => match env_copy.throw(jni::objects::JThrowable::from(obj)) {
										Err(e) => panic!("error throwing Java exception -- failed throwing: {e}"),
										Ok(_) => return #return_expr,
									},
								},
							},
						}
						Ok(ret) => ret,
					}
				}
			}
			// ^----------------------------------^
		}
	} else {
		// V----------------------------------V
		quote::quote! {
			#[no_mangle]
			#[allow(unused_mut)]
			pub extern "system" fn #fn_name<'local>(#incoming) #ret_type {
				#fn_name_inner(#forwarding)
			}
		}
		// ^----------------------------------^
	};

	out.append_all(input);
	out.append_all(wrapped);
	Ok(out)
}

enum WhatNext {
	Nothing,
	Package,
	Class,
	Exception,
}

fn unpack_pat(pat: syn::Pat) -> Result<TokenStream, syn::Error> {
	match pat {
		syn::Pat::Ident(i) => {
			let ident = i.ident;
			Ok(quote::quote!( #ident ,))
		},
		syn::Pat::Reference(r) => {
			unpack_pat(*r.pat)
		},
		_ => Err(syn::Error::new(Span::call_site(), "unsupported argument type")),
	}
}

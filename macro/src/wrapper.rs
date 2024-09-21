use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;
use syn::{FnArg, Item, ReturnType, Type};

use crate::attrs::AttrsOptions;

pub(crate) fn generate_jni_wrapper(attrs: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
	let mut out = TokenStream::new();

	let Item::Fn(fn_item) = syn::parse2(input.clone())? else {
		return Err(syn::Error::new(Span::call_site(), "#[jni] is only supported on functions"));
	};

	let attrs = AttrsOptions::parse_attr(attrs)?;

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
	let fn_name = syn::Ident::new(&format!("Java_{}_{}_{name_jni}", attrs.package, attrs.class), Span::call_site());

	let Some(env_ident) = forwarding.clone().into_iter().next() else {
		return Err(syn::Error::new(Span::call_site(), "missing JNIEnv argument"));
	};

	let return_expr = if attrs.return_pointer {
		quote::quote!( std::ptr::null_mut() )
	} else {
		quote::quote!( 0 )
	};

	let wrapped = if could_error {
		if let Some(exception) = attrs.exception {
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

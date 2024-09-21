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

	let item = syn::parse2(input.clone())?;

	let Item::Fn(fn_item) = item else {
		panic!("can only be applied to functions"); // TODO throw err instead of panic
	};

	let mut what_next = WhatNext::Nothing;

	let mut package = None;
	let mut clazz = None;

	for attr in attrs {
		match what_next {
			WhatNext::Nothing => {
				if let TokenTree::Ident(ref i) = attr {
					match i.to_string().as_ref() {
						"package" => what_next = WhatNext::Package,
						"class" => what_next = WhatNext::Class,
						_ => panic!("unexpected keyword in attrs: {}", attr),
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
		}
	}

	let package = package.expect("missing attribute 'package'");
	let clazz = clazz.expect("missing attribute 'class'");

	let (could_error, ret_type) = match fn_item.sig.output {
		syn::ReturnType::Default => (false, fn_item.sig.output),
		syn::ReturnType::Type(_tok, ty) => match *ty {
			syn::Type::Path(ref path) => {
				let Some(last) = path.path.segments.last() else {
					panic!("empty type path");
				};

				// TODO this is terrible, macro returns a function and we call it?? there must be a
				// better way!!!
				let mut out = (
					false,
					ReturnType::Type(syn::Token![->](Span::call_site()), Box::new(Type::Path(path.clone())))
				);

				if last.ident == "Result" {
					match &last.arguments {
						syn::PathArguments::None => panic!("result without generics is not valid"),
						syn::PathArguments::Parenthesized(_) => panic!("parenthesized result is not valid"),
						syn::PathArguments::AngleBracketed(ref generics) => for generic in generics.args.iter() {
							match generic {
								syn::GenericArgument::Lifetime(_) => continue,
								syn::GenericArgument::Type(ty) => {
									out = (true, ReturnType::Type(syn::Token![->](Span::call_site()), Box::new(ty.clone())));
									break;
								},
								_ => panic!("unexpected type in Result generic"),
							}
						}
					}
				}

				out
			},
			_ => panic!("unsupported return type"),
		},
	};


	let mut incoming = TokenStream::new();
	let mut forwarding = TokenStream::new();

	for arg in fn_item.sig.inputs {
		let FnArg::Typed(ty) = arg else {
			panic!("jni macro doesn't work on methods");
		};
		incoming.append_all(quote::quote!( #ty , ));
		let pat = ty.pat;
		forwarding.append_all(quote::quote!( #pat , ));
	}

	let name = fn_item.sig.ident.to_string();
	let name_jni = name.replace("_", "_1");
	let fn_name_inner = syn::Ident::new(&name, Span::call_site());
	let fn_name = syn::Ident::new(&format!("Java_{package}_{clazz}_{name_jni}"), Span::call_site());

	let wrapped = if could_error {
		quote::quote! {
			#[no_mangle]
			pub extern "system" fn #fn_name<'local>(#incoming) #ret_type {
				match #fn_name_inner(#forwarding) {
					Ok(x) => x,
					Err(e) => panic!("error in JNI!"), // TODO throw java exc
				}
			}
		}
	} else {
		quote::quote! {
			#[no_mangle]
			pub extern "system" fn #fn_name<'local>(#incoming) #ret_type {
				#fn_name_inner(#forwarding)
			}
		}
	};

	out.append_all(input);
	out.append_all(wrapped);
	Ok(out)
}

enum WhatNext {
	Nothing,
	Package,
	Class,
}

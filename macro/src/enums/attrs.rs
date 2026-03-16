use proc_macro2::{Span, TokenStream, TokenTree};

pub(crate) struct AttrsOptions {
	pub(crate) clazz: String,
	pub(crate) package: String,
}

impl AttrsOptions {
	pub(crate) fn parse_attr(attrs: TokenStream, attr_enum: &syn::ItemEnum) -> Result<Self, syn::Error> {
		let mut what_next = WhatNext::Nothing;

		let mut package = None;
		let mut clazz = None;
	
		for attr in attrs {
			match what_next {
				WhatNext::Nothing => {
					if let TokenTree::Ident(ref i) = attr {
						match i.to_string().as_ref() {
							"class" => what_next = WhatNext::Class,
							"package" => what_next = WhatNext::Package,
							val => return Err(syn::Error::new(Span::call_site(), format!("unexpected attribute on macro: {val}"))),
						}
					}
				},
				WhatNext::Class => {
					if let TokenTree::Literal(i) = attr {
						clazz = Some(i.to_string().replace('"', ""));
						what_next = WhatNext::Nothing;
					}
				},
				WhatNext::Package => {
					if let TokenTree::Literal(i) = attr {
						package = Some(i.to_string().replace('"', "").replace(".", "/"));
						what_next = WhatNext::Nothing;
					}
				},
			}
		}

		let Some(package) = package else {
			return Err(syn::Error::new(Span::call_site(), "missing required attribute 'package'"))
		};

		Ok(Self { clazz: clazz.unwrap_or(attr_enum.ident.to_string()), package })
	}
}

enum WhatNext {
	Nothing,
	Class,
	Package,
}

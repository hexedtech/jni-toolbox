use proc_macro2::{Span, TokenStream, TokenTree};

pub(crate) struct AttrsOptions {
	pub(crate) clazz: String,
}

impl AttrsOptions {
	pub(crate) fn parse_attr(attrs: TokenStream) -> Result<Self, syn::Error> {
		let mut what_next = WhatNext::Nothing;

		let mut clazz = None;
	
		for attr in attrs {
			match what_next {
				WhatNext::Nothing => {
					if let TokenTree::Ident(ref i) = attr {
						match i.to_string().as_ref() {
							"class" => what_next = WhatNext::Class,
							_ => return Err(syn::Error::new(Span::call_site(), "unexpected attribute on macro: {attr}")),
						}
					}
				},
				WhatNext::Class => {
					if let TokenTree::Literal(i) = attr {
						clazz = Some(i.to_string().replace('"', ""));
						what_next = WhatNext::Nothing;
					}
				},
			}
		}

		let Some(clazz) = clazz else { return Err(syn::Error::new(Span::call_site(), "missing required attribute 'class'")) };

		Ok(Self { clazz })
	}
}

enum WhatNext {
	Nothing,
	Class,
}

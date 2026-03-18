use proc_macro2::{Span, TokenStream, TokenTree};

pub(crate) struct AttrsOptions {
	pub(crate) package: Option<String>,
	pub(crate) class: Option<String>,
	pub(crate) inline: Option<bool>,
}

impl AttrsOptions {
	pub(crate) fn parse_attr(attrs: TokenStream) -> Result<Self, syn::Error> {
		let mut what_next = WhatNext::Nothing;

		let mut package = None;
		let mut class = None;
		let mut inline = None;
	
		for attr in attrs {
			match what_next {
				WhatNext::Nothing => {
					if let TokenTree::Ident(ref i) = attr {
						match i.to_string().as_ref() {
							"package" => what_next = WhatNext::Package,
							"class" => what_next = WhatNext::Class,
							"exception" => what_next = WhatNext::Exception,
							"ptr" => {}, // accepted for backwards compatibility
							"inline" => inline = Some(true),
							val => return Err(syn::Error::new(Span::call_site(), format!("unexpected attribute on macro: {val}"))),
						}
					}
				},
				WhatNext::Class => {
					if let TokenTree::Literal(i) = attr {
						class = Some(i.to_string().replace('"', ""));
						what_next = WhatNext::Nothing;
					}
				},
				WhatNext::Package => {
					if let TokenTree::Literal(i) = attr {
						package = Some(i.to_string().replace('"', "").replace(".", "_"));
						what_next = WhatNext::Nothing;
					}
				},
				WhatNext::Exception => {
					if let TokenTree::Literal(_i) = attr {
						what_next = WhatNext::Nothing;
					}
				}
			}
		}

		Ok(Self { package, class, inline })
	}
}

enum WhatNext {
	Nothing,
	Package,
	Class,
	Exception,
}

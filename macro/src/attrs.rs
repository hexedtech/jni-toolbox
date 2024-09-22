use proc_macro2::{Span, TokenStream, TokenTree};

pub(crate) struct AttrsOptions {
	pub(crate) package: String,
	pub(crate) class: String,
	pub(crate) exception: Option<String>,
	pub(crate) return_pointer: bool,
}

impl AttrsOptions {
	pub(crate) fn parse_attr(attrs: TokenStream) -> Result<Self, syn::Error> {
		let mut what_next = WhatNext::Nothing;

		let mut package = None;
		let mut class = None;
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
					if let TokenTree::Literal(i) = attr {
						exception = Some(i.to_string().replace('"', "").replace(".", "_"));
						what_next = WhatNext::Nothing;
					}
				}
			}
		}

		let Some(package) = package else { return Err(syn::Error::new(Span::call_site(), "missing required attribute 'package'")) };
		let Some(class) = class else { return Err(syn::Error::new(Span::call_site(), "missing required attribute 'class'")) };

		Ok(Self { package, class, exception, return_pointer })
	}
}

enum WhatNext {
	Nothing,
	Package,
	Class,
	Exception,
}

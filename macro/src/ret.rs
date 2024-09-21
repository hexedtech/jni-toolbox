use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{ReturnType, Type};

#[derive(Clone)]
pub(crate) struct ReturnOptions {
	pub(crate) ty: Option<Box<Type>>,
	pub(crate) result: bool,
}

impl ReturnOptions {
	pub(crate) fn parse_signature(ret: ReturnType) -> Result<Self, syn::Error> {
		match ret {
			syn::ReturnType::Default => Ok(Self { ty: None, result: false }),
			syn::ReturnType::Type(_tok, ty) => match *ty {
				syn::Type::Path(ref path) => {
					let Some(last) = path.path.segments.last() else {
						return Err(syn::Error::new(Span::call_site(), "empty Result type is not valid"));
					};

					if last.ident == "Result" {
						match &last.arguments {
							syn::PathArguments::None => return Err(syn::Error::new(Span::call_site(), "Result without generics is not valid")),
							syn::PathArguments::Parenthesized(_) => return Err(syn::Error::new(Span::call_site(), "Parenthesized Result is not valid")),
							syn::PathArguments::AngleBracketed(ref generics) => for generic in generics.args.iter() {
								match generic {
									syn::GenericArgument::Lifetime(_) => continue,
									syn::GenericArgument::Type(ty) => return Ok(Self { ty: Some(Box::new(ty.clone())), result: true }),
									_ => return Err(syn::Error::new(Span::call_site(), "unexpected type in Result")),
								}
							}
						}
					}

					Ok(Self { ty: Some(Box::new(Type::Path(path.clone()))), result: false })
				},
				_ => Err(syn::Error::new(Span::call_site(), "unsupported return type")),
			},
		}
	}

	pub(crate) fn tokens(self) -> TokenStream {
		match self.ty {
			Some(t) => ReturnType::Type(syn::Token![->](Span::call_site()), t).to_token_stream(),
			None => ReturnType::Default.to_token_stream(),
		}
	}
}

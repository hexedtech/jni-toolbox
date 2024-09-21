use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{ReturnType, Type};

#[derive(Clone)]
pub(crate) struct ReturnOptions {
	pub(crate) ty: Option<Box<Type>>,
	pub(crate) result: bool,
	pub(crate) void: bool,
}

impl ReturnOptions {
	pub(crate) fn parse_signature(ret: &ReturnType) -> Result<Self, syn::Error> {
		match ret {
			syn::ReturnType::Default => Ok(Self { ty: None, result: false, void: true }),
			syn::ReturnType::Type(_tok, ty) => match *ty.clone() {
				syn::Type::Path(path) => {
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
									syn::GenericArgument::Type(ty) => return Ok(Self { ty: Some(Box::new(ty.clone())), result: true, void: is_void(ty) }),
									_ => return Err(syn::Error::new(Span::call_site(), "unexpected type in Result")),
								}
							}
						}
					}

					Ok(Self { ty: Some(Box::new(Type::Path(path.clone()))), result: false, void: false })
				},
				_ => Err(syn::Error::new(Span::call_site(), "unsupported return type")),
			},
		}
	}

	pub(crate) fn tokens(&self) -> TokenStream {
		match &self.ty { // TODO why do we need to invoke syn::Token! macro ???
			Some(t) => quote::quote!( -> <#t as jni_toolbox::IntoJava<'local>>::T ),
			None => ReturnType::Default.to_token_stream(),
		}
	}
}

fn is_void(ty: &syn::Type) -> bool {
	match ty {
		Type::Array(_) => false,
		Type::BareFn(_) => false,
		Type::Group(g) => is_void(&g.elem),
		Type::ImplTrait(_) => false,
		Type::Infer(_) => false,
		Type::Macro(_) => false,
		Type::Never(_) => true,
		Type::Paren(p) => is_void(&p.elem),
		Type::Path(p) => p.path.segments.is_empty(),
		Type::Ptr(_) => false,
		Type::Reference(_) => false,
		Type::Slice(_) => false,
		Type::TraitObject(_) => false,
		Type::Tuple(x) => x.elems.is_empty(),
		Type::Verbatim(_) => false,
		_ => todo!(),
	}
}

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::Ident;

pub(crate) struct ArgumentOptions {
	pub(crate) incoming: TokenStream,
	pub(crate) transforming: TokenStream,
	pub(crate) forwarding: TokenStream,
	pub(crate) env: Ident,
}

fn unpack_pat(pat: syn::Pat) -> Result<TokenStream, syn::Error> {
	match pat {
		syn::Pat::Ident(i) => {
			let ident = i.ident;
			Ok(quote::quote!( #ident ))
		},
		syn::Pat::Reference(r) => {
			unpack_pat(*r.pat)
		},
		_ => Err(syn::Error::new(Span::call_site(), "unsupported argument type")),
	}
}

fn type_equals(ty: Box<syn::Type>, search: impl AsRef<str>) -> bool {
	match *ty {
		syn::Type::Array(_) => false,
		syn::Type::BareFn(_) => false,
		syn::Type::ImplTrait(_) => false,
		syn::Type::Infer(_) => false,
		syn::Type::Macro(_) => false,
		syn::Type::Never(_) => false,
		syn::Type::Ptr(_) => false,
		syn::Type::Slice(_) => false,
		syn::Type::TraitObject(_) => false,
		syn::Type::Tuple(_) => false,
		syn::Type::Verbatim(_) => false,
		syn::Type::Group(g) => type_equals(g.elem, search),
		syn::Type::Paren(p) => type_equals(p.elem, search),
		syn::Type::Reference(r) => type_equals(r.elem, search),
		syn::Type::Path(ty) => {
			ty.path.segments
				.last()
				.map_or(false, |e| e.ident == search.as_ref())
		},
		_ => false,
	}
}

impl ArgumentOptions {
	pub(crate) fn parse_args(fn_item: &syn::ItemFn, ret_expr: TokenStream) -> Result<Self, syn::Error> {
		let mut arguments = Vec::new();
		let mut pass_env = false;
		let mut pass_class = false;
		for arg in fn_item.sig.inputs.iter() {
			let syn::FnArg::Typed(ty) = arg else {
				return Err(syn::Error::new(Span::call_site(), "#[jni] macro doesn't work on methods"));
			};
			let pat = unpack_pat(*ty.pat.clone())?;
			if type_equals(ty.ty.clone(), "JNIEnv") { pass_env = true };
			if type_equals(ty.ty.clone(), "JClass") { pass_class = true };
			arguments.push(SingleArgument {
				pat: syn::Ident::new(&pat.to_string(), Span::call_site()),
				ty: ty.ty.clone(),
			});
		}

		let mut incoming = TokenStream::new();
		let mut transforming = TokenStream::new();
		let mut forwarding = TokenStream::new();

		let env = if pass_env {
			arguments.first()
				.ok_or_else(|| syn::Error::new(Span::call_site(), "missing env parameter"))?
				.pat
				.clone()
		} else {
			syn::Ident::new("env", Span::call_site())
		};

		let mut args_iter = arguments.into_iter();
		
		if pass_env {
			if let Some(arg) = args_iter.next() {
				let pat = arg.pat;
				let ty = arg.ty;
				incoming.append_all(quote::quote!( mut #pat: #ty,));
				forwarding.append_all(quote::quote!( #pat,));
			}
		} else {
			incoming.append_all(quote::quote!( mut #env: jni::JNIEnv<'local>,));
		}

		if !pass_class {
			incoming.append_all(quote::quote!( _class: jni::objects::JClass<'local>,));
		}

		for arg in args_iter {
			let pat = arg.pat;
			let new_pat = syn::Ident::new(&format!("{pat}_new"), Span::call_site());
			let ty = arg.ty;
			transforming.append_all(quote::quote!{
				let #new_pat = match jni_toolbox::from_java_static::<#ty>(&mut #env, #pat) {
					Ok(x) => x,
					Err(e) => {
						// TODO should we panic here instead?
						let _ = #env.throw_new("java/lang/RuntimeException", format!("{e:?}"));
						return #ret_expr;
					},
				};
			});
			incoming.append_all(quote::quote!( #pat: <#ty as jni_toolbox::FromJava<'local>>::From,));
			forwarding.append_all(quote::quote!( #new_pat,));
		}

		Ok(Self { incoming, transforming, forwarding, env })
	}
}

struct SingleArgument {
	pat: syn::Ident,
	ty: Box<syn::Type>,
}

#[allow(unused)]
fn bare_type(t: syn::Type) -> TokenStream {
	match t {
		syn::Type::Array(x) => bare_type(*x.elem),
		syn::Type::BareFn(f) => f.to_token_stream(),
		syn::Type::Group(x) => bare_type(*x.elem),
		syn::Type::ImplTrait(t) => t.to_token_stream(),
		syn::Type::Infer(x) => x.to_token_stream(),
		syn::Type::Macro(x) => x.to_token_stream(),
		syn::Type::Never(x) => x.to_token_stream(),
		syn::Type::Paren(p) => bare_type(*p.elem),
		syn::Type::Path(p) => p.to_token_stream(),
		syn::Type::Ptr(x) => bare_type(*x.elem),
		syn::Type::Reference(r) => bare_type(*r.elem),
		syn::Type::Slice(s) => bare_type(*s.elem),
		syn::Type::TraitObject(t) => t.to_token_stream(),
		syn::Type::Tuple(x) => x.to_token_stream(),
		syn::Type::Verbatim(x) => x.to_token_stream(),
		_ => todo!(),
	}
}

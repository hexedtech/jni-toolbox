use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;
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

fn bare_type(ty: Box<syn::Type>) -> Option<syn::TypePath> {
	match *ty {
		syn::Type::Array(a) => bare_type(a.elem),
		syn::Type::BareFn(_) => None,
		syn::Type::ImplTrait(_) => None,
		syn::Type::Infer(_) => None,
		syn::Type::Macro(_) => None,
		syn::Type::Never(_) => None,
		syn::Type::TraitObject(_) => None,
		syn::Type::Verbatim(_) => None,
		syn::Type::Ptr(p) => bare_type(p.elem),
		syn::Type::Slice(s) => bare_type(s.elem),
		syn::Type::Tuple(t) => bare_type(Box::new(t.elems.first()?.clone())), // TODO
		syn::Type::Group(g) => bare_type(g.elem),
		syn::Type::Paren(p) => bare_type(p.elem),
		syn::Type::Reference(r) => bare_type(r.elem),
		syn::Type::Path(ty) => Some(ty),
		_ => todo!(),
	}
}

fn type_equals(ty: Box<syn::Type>, search: impl AsRef<str>) -> bool {
	let Some(ty) = bare_type(ty) else { return false };
	let Some(last) = ty.path.segments.last() else { return false };
	last.ident == search.as_ref()
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
				let ty = bare_type(arg.ty);
				incoming.append_all(quote::quote!( mut #pat: #ty,));
				forwarding.append_all(quote::quote!( &mut #pat,));
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
						let _ = #env.throw_new(e.jclass(), format!("{e:?}"));
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

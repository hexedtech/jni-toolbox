use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;


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

pub(crate) fn parse_args(fn_item: syn::ItemFn) -> Result<(TokenStream, TokenStream), syn::Error> {
	let mut incoming = TokenStream::new();
	let mut forwarding = TokenStream::new();
	for arg in fn_item.sig.inputs {
		let syn::FnArg::Typed(ty) = arg else {
			return Err(syn::Error::new(Span::call_site(), "#[jni] macro doesn't work on methods"));
		};
		incoming.append_all(quote::quote!( #ty , ));
		let pat = unpack_pat(*ty.pat)?;
		forwarding.append_all(pat);
	}
	Ok((incoming, forwarding))
}

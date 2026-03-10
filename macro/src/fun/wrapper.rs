use proc_macro2::{Span, TokenStream};
use syn::Item;

use crate::fun::{args::ArgumentOptions, attrs::AttrsOptions, ret::ReturnOptions};

pub(crate) fn generate_jni_wrapper(attrs: TokenStream, original_fn: TokenStream) -> Result<TokenStream, syn::Error> {
	let Item::Fn(fn_item) = syn::parse2(original_fn.clone())? else {
		return Err(syn::Error::new(Span::call_site(), "#[jni] is only supported on functions"));
	};

	let attrs = AttrsOptions::parse_attr(attrs)?;
	let ret = ReturnOptions::parse_signature(&fn_item.sig.output)?;

	// TODO a bit ugly passing the return expr down... we should probably manage returns here
	let args = ArgumentOptions::parse_args(&fn_item)?;

	let return_type = ret.tokens();

	let name = fn_item.sig.ident.to_string();
	let name_jni = name.replace("_", "_1");
	let fn_name_inner = syn::Ident::new(&name, Span::call_site());
	let fn_name = syn::Ident::new(&format!("Java_{}_{}_{name_jni}", attrs.package, attrs.class), Span::call_site());

	let incoming = args.incoming;
	// V----------------------------------V
	let header = quote::quote! {
		#[no_mangle]
		#[allow(unused_unit, non_snake_case)]
		pub extern "system" fn #fn_name<'local>(#incoming) #return_type
	};

	let transforming = args.transforming;
	let env_iden = args.env;
	let forwarding = args.forwarding;

	let inline_macro = if attrs.inline {
		quote::quote!(#[inline])
	} else {
		quote::quote!()
	};

	let error_handling = if ret.result {
		quote::quote!( let ret = result?; )
	} else {
		quote::quote!( let ret = result; )
	};

	Ok(quote::quote! {
		#inline_macro
		#original_fn

		#header {
			env.with_env(|mut env| -> Result<_, jni_toolbox::Error> {
				use jni_toolbox::{FromJava, IntoJava};

				#transforming

				let result = #fn_name_inner(#forwarding);

				#error_handling
				
				Ok(ret.into_java(&mut #env_iden)?)
			})
				.resolve::<jni_toolbox::ErrorPolicy>()
		}
	})
}

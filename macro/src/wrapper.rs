use proc_macro2::{Span, TokenStream};
use syn::Item;

use crate::{args::ArgumentOptions, attrs::AttrsOptions, ret::ReturnOptions};

pub(crate) fn generate_jni_wrapper(attrs: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
	let Item::Fn(fn_item) = syn::parse2(input.clone())? else {
		return Err(syn::Error::new(Span::call_site(), "#[jni] is only supported on functions"));
	};

	let attrs = AttrsOptions::parse_attr(attrs)?;
	let ret = ReturnOptions::parse_signature(&fn_item.sig.output)?;
	let return_expr = if ret.void {
		quote::quote!( () )
	} else if ret.pointer {
		quote::quote!( std::ptr::null_mut() )
	} else {
		quote::quote!( 0 )
	};

	// TODO a bit ugly passing the return expr down... we should probably manage returns here
	let args = ArgumentOptions::parse_args(&fn_item, return_expr.clone())?;

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
	let transformations = quote::quote! {
		use jni_toolbox::{JniToolboxError, FromJava, IntoJava};
		#transforming
	};


	let env_iden = args.env;
	let forwarding = args.forwarding;
	let invocation = quote::quote! {
		let result = #fn_name_inner(#forwarding);
	};


	let error_handling = if ret.result {
		if let Some(exception) = attrs.exception {
			quote::quote! {
				let ret = match result {
					Ok(x) => x,
					Err(e) => match #env_iden.throw_new(#exception, format!("{e:?}")) {
						Ok(_) => return #return_expr,
						Err(e) => panic!("error throwing java exception: {e}"),
					}
				};
			}
		} else {
			quote::quote! {
				let ret = match result {
					Ok(x) => x,
					Err(e) => match #env_iden.throw_new(e.jclass(), format!("{e:?}")) {
						Err(e) => panic!("error throwing Java exception -- failed throwing: {e}"),
						Ok(_) => return #return_expr
					}
				};
			}
		}
	} else {
		quote::quote!( let ret = result; )
	};


	let reverse_transformations = quote::quote! {
		match ret.into_java(&mut #env_iden) {
			Ok(fin) => fin,
			Err(e) => {
				// TODO should we panic instead?
				let _ = #env_iden.throw_new(e.jclass(), format!("{e:?}"));
				#return_expr
			}
		}
	};

	let inline_macro = if attrs.inline {
		quote::quote!(#[inline])
	} else {
		quote::quote!()
	};

	Ok(quote::quote! {
		#inline_macro
		#input

		#header {

			#transformations

			#invocation

			#error_handling

			#reverse_transformations

		}
	})
}

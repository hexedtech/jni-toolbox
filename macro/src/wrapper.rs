use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;
use syn::Item;

use crate::{args::ArgumentOptions, attrs::AttrsOptions, ret::ReturnOptions};

pub(crate) fn generate_jni_wrapper(attrs: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
	let mut out = TokenStream::new();

	let Item::Fn(fn_item) = syn::parse2(input.clone())? else {
		return Err(syn::Error::new(Span::call_site(), "#[jni] is only supported on functions"));
	};

	let attrs = AttrsOptions::parse_attr(attrs)?;
	let ret = ReturnOptions::parse_signature(&fn_item.sig.output)?;
	let return_expr = if ret.ty.is_none() {
		quote::quote!( () )
	} else if attrs.return_pointer {
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
		#[allow(unused_mut)]
		pub extern "system" fn #fn_name<'local>(#incoming) #return_type
	};
	// ^----------------------------------^

	let env_ident = args.env;
	let forwarding = args.forwarding;
	let transforming = args.transforming;
	let body = if ret.result { // wrap errors
		if let Some(exception) = attrs.exception {
			// V----------------------------------V
			quote::quote! {
				{
					use jni_toolbox::{JniToolboxError, FromJava};
					#transforming
					match #fn_name_inner(#forwarding) {
						Ok(ret) => ret,
						Err(e) => match #env_ident.throw_new(#exception, format!("{e:?}")) {
							Ok(_) => return #return_expr,
							Err(e) => panic!("error throwing java exception: {e}"),
						}
					}
				}
			}
			// ^----------------------------------^
		} else {
			// V----------------------------------V
			quote::quote! {
				{
					use jni_toolbox::{JniToolboxError, FromJava};
					// NOTE: this should be SAFE! the cloned env reference lives less than the actual one, we just lack a
					//       way to get it back from the called function and thus resort to unsafe cloning
					let mut env_copy = unsafe { #env_ident.unsafe_clone() };
					#transforming
					match #fn_name_inner(#forwarding) {
						Err(e) => match env_copy.find_class(e.jclass()) {
							Err(e) => panic!("error throwing Java exception -- failed resolving error class: {e}"),
							Ok(class) => match env_copy.new_string(format!("{e:?}")) {
								Err(e) => panic!("error throwing Java exception --  failed creating error string: {e}"),
								Ok(msg) => match env_copy.new_object(class, "(Ljava/lang/String;)V", &[jni::objects::JValueGen::Object(&msg)]) {
									Err(e) => panic!("error throwing Java exception -- failed creating object: {e}"),
									Ok(obj) => match env_copy.throw(jni::objects::JThrowable::from(obj)) {
										Err(e) => panic!("error throwing Java exception -- failed throwing: {e}"),
										Ok(_) => return #return_expr,
									},
								},
							},
						}
						Ok(ret) => ret,
					}
				}
			}
		}
	} else {
		// V----------------------------------V
		quote::quote! {
			{
				use jni_toolbox::{JniToolboxError, FromJava};
				#transforming
				#fn_name_inner(#forwarding)
			}
		}
		// ^----------------------------------^
	};

	out.append_all(input);
	out.append_all(header);
	out.append_all(body);
	Ok(out)
}

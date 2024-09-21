use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;
use syn::Item;

use crate::{args::parse_args, attrs::AttrsOptions, ret::ReturnOptions};

pub(crate) fn generate_jni_wrapper(attrs: TokenStream, input: TokenStream) -> Result<TokenStream, syn::Error> {
	let mut out = TokenStream::new();

	let Item::Fn(fn_item) = syn::parse2(input.clone())? else {
		return Err(syn::Error::new(Span::call_site(), "#[jni] is only supported on functions"));
	};

	let attrs = AttrsOptions::parse_attr(attrs)?;
	let ret = ReturnOptions::parse_signature(fn_item.sig.output.clone())?;


	let return_type = ret.clone().tokens();
	let name = fn_item.sig.ident.to_string();
	let name_jni = name.replace("_", "_1");
	let fn_name_inner = syn::Ident::new(&name, Span::call_site());
	let fn_name = syn::Ident::new(&format!("Java_{}_{}_{name_jni}", attrs.package, attrs.class), Span::call_site());

	let (incoming, forwarding) = parse_args(fn_item)?;

	let header = quote::quote! {

		#[no_mangle]
		#[allow(unused_mut)]
		pub extern "system" fn #fn_name<'local>(#incoming) #return_type

	};

	let return_expr = if attrs.return_pointer {
		quote::quote!( std::ptr::null_mut() )
	} else {
		quote::quote!( 0 )
	};

	let Some(env_ident) = forwarding.clone().into_iter().next() else {
		return Err(syn::Error::new(Span::call_site(), "missing JNIEnv argument"));
	};


	let body = if ret.result { // wrap errors
		if let Some(exception) = attrs.exception {
			quote::quote! {
				{
					use jni_toolbox::JniToolboxError;
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
					use jni_toolbox::JniToolboxError;
					// NOTE: this is SAFE! the cloned env reference lives less than the actual one, we just lack a
					//       way to get it back from the called function and thus resort to unsafe cloning
					let mut env_copy = unsafe { #env_ident.unsafe_clone() };
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

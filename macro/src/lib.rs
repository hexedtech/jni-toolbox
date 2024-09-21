mod attrs;
mod wrapper;


/// wrap this function in in a JNI exported fn
#[proc_macro_attribute]
pub fn jni(
	attrs: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	wrapper::generate_jni_wrapper(
		syn::parse_macro_input!(attrs),
		syn::parse_macro_input!(input),
	)
		.unwrap()
		.into()
}

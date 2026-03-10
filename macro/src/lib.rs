mod ext;
mod fun;
mod clazz;

/// Wrap this function in in a JNI exported fn.
#[proc_macro_attribute]
pub fn jni(
	attrs: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	fun::wrapper::generate_jni_wrapper(
		syn::parse_macro_input!(attrs),
		syn::parse_macro_input!(input),
	)
		.unwrap()
		.into()
}

/// Implement IntoJavaObject and FromJavaObject for this struct
#[proc_macro_attribute]
pub fn jclass(
	attrs: proc_macro::TokenStream,
	input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
	clazz::wrapper::generate_jobject_conversions(
		syn::parse_macro_input!(attrs),
		syn::parse_macro_input!(input),
	)
		.unwrap()
		.into()
}

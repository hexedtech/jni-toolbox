use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;

pub(crate) fn generate_jobject_conversions(attrs: TokenStream, original_struct: TokenStream) -> Result<TokenStream, syn::Error> {
	let syn::Item::Struct(s) = syn::parse2(original_struct.clone())? else {
		return Err(syn::Error::new(Span::call_site(), "#[jclass] is only supported on structs"));
	};

	let attrs = super::attrs::AttrsOptions::parse_attr(attrs)?;

	let mut builder_fields = TokenStream::new();
	let mut getter_fields = TokenStream::new();
	let mut constructor_fields = TokenStream::new();
	let mut constructor_args = TokenStream::new();
	let mut transformations = TokenStream::new();

	for (_i, f) in s.fields.iter().enumerate() {
		if let Some(name) = f.ident.clone() {
			let name_str = stringify!(name);
			let ty = f.ty.clone();
			let sig = super::maps::sig(ty.clone());

			builder_fields.append_all(quote::quote!( #name, ));

			getter_fields.append_all(quote::quote!(
				let #name = {
					let ___field = env.get_field(&object, jni::jni_str!(#name_str), jni::jni_sig!(#sig))?;
					#ty::from_jvalue(env, ___field)?
				};
			));

			transformations.append_all(quote::quote!(
				let #name = self.#name.into_jvalue(env)?;
			));

			constructor_fields.append_all(quote::quote!( arg : #sig,));

			constructor_args.append_all(quote::quote!( #name.borrow(), ));
		}
	}

	let builder = quote::quote!(
		Ok(Self { #builder_fields })
	);
	let constructor = quote::quote! (
		( #constructor_fields ) -> ()
	);

	let struct_type = s.ident;
	let clazz = attrs.clazz;

	Ok(quote::quote! {
		#original_struct

		impl<'local> jni_toolbox::IntoJavaObject<'local> for #struct_type {
			const CLASS: &'static str = #clazz;
			fn into_java_object(
				self,
				env: &mut jni::Env<'local>,
			) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
				use jni_toolbox::IntoJava;
				let ___clazz = env.find_class(jni::strings::JNIString::new(Self::CLASS))?;

				#transformations;

				env.new_object(
					___clazz,
					jni::jni_sig!(#constructor),
					&[
						#constructor_args
					],
				)
			}
		}

		impl<'local> jni_toolbox::FromJava<'local> for #struct_type {
			type From = jni::objects::JObject<'local>;
			fn from_java(
				env: &mut jni::Env<'local>,
				object: Self::From,
			) -> Result<Self, jni::errors::Error> {
				use jni_toolbox::FromJava;

				#getter_fields

				#builder
			}

			fn from_jvalue(
				env: &mut jni::Env<'local>,
				value: jni::JValueOwned<'local>,
			) -> Result<Self, jni::errors::Error> {
				todo!()
			}
		}
	})
}

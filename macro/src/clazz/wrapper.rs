use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;

pub(crate) fn generate_jobject_conversions(attrs: TokenStream, original_struct: TokenStream) -> Result<TokenStream, syn::Error> {
	let syn::Item::Struct(s) = syn::parse2(original_struct.clone())? else {
		return Err(syn::Error::new(Span::call_site(), "#[jclass] is only supported on structs"));
	};

	let attrs = super::attrs::AttrsOptions::parse_attr(attrs, &s)?;

	let mut builder_fields = TokenStream::new();
	let mut getter_fields = TokenStream::new();
	let mut constructor_args = TokenStream::new();
	let mut transformations = TokenStream::new();
	let mut constructor_str= TokenStream::new();
	let mut constructor_types = TokenStream::new();

	for f in s.fields.iter() {
		if let Some(name) = f.ident.clone() {
			let name_str = stringify!(name);
			let ty = f.ty.clone();

			constructor_str.append_all(quote::quote! (
				<#ty as IntoJava>::signature().0.as_str() +
			));
			constructor_types.append_all(quote::quote!(
				<#ty as IntoJava>::signature().1,
			));

			builder_fields.append_all(quote::quote!( #name, ));

			getter_fields.append_all(quote::quote!(
				let #name = {
					let (sig, ty) = <#ty as IntoJava>::signature();
					let sig_jni = jni::strings::JNIString::new(sig);
					let ___sig = unsafe {
						jni::signature::FieldSignature::from_raw_parts(
							sig_jni.borrowed(),
							ty,
						)
					};
					let ___field = env.get_field(&object, jni::jni_str!(#name_str), ___sig)?;
					jni_toolbox::from_jvalue_static::<#ty>(env, ___field)?
				};
			));

			transformations.append_all(quote::quote!(
				let #name = self.#name.into_jvalue(env)?;
			));

			constructor_args.append_all(quote::quote!( #name.borrow(), ));
		}
	}

	let builder = quote::quote!(
		Ok(Self { #builder_fields })
	);
	let constructor_str_concat = quote::quote! (
		String::new() + #constructor_str ""
	);

	let struct_type = s.ident;
	let clazz = format!("{}/{}", attrs.package, attrs.clazz);

	Ok(quote::quote! {
		#original_struct

		impl<'local> jni_toolbox::IntoJavaObject<'local> for #struct_type {
			const CLASS: &'static str = #clazz;
			fn into_java_object(
				self,
				env: &mut jni::Env<'local>,
			) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
				use jni_toolbox::{IntoJava, FromJava};
				let ___clazz = env.find_class(jni::strings::JNIString::new(Self::CLASS))?;

				#transformations;

				let ___sig_jni = jni::strings::JNIString::new(#constructor_str_concat);
				let ___sig_tys = [
					#constructor_types
				];
				let ___ctor_sig = unsafe {
					jni::signature::MethodSignature::from_raw_parts(
						___sig_jni.borrowed(),
						&___sig_tys,
						jni::signature::JavaType::Primitive(jni::signature::Primitive::Void),
					)
				};

				env.new_object(
					___clazz,
					___ctor_sig,
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
				use jni_toolbox::{IntoJava, FromJava};

				#getter_fields

				#builder
			}

			fn from_jvalue(
				env: &mut jni::Env<'local>,
				value: jni::JValueOwned<'local>,
			) -> Result<Self, jni::errors::Error> {
				Self::from_java(env, value.l()?)
			}
		}
	})
}

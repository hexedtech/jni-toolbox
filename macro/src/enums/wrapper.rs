use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;

pub(crate) fn generate_enum_conversions(attrs: TokenStream, original_enum: TokenStream) -> Result<TokenStream, syn::Error> {
	let syn::Item::Enum(e) = syn::parse2(original_enum.clone())? else {
		return Err(syn::Error::new(Span::call_site(), "#[jenum] is only supported on c-like enums"));
	};

	let attrs = super::attrs::AttrsOptions::parse_attr(attrs, &e)?;

	let mut get_java_variant = TokenStream::new();
	let mut get_rust_variant = TokenStream::new();

	for (o, v) in e.variants.iter().enumerate() {
		if !matches!(v.fields, syn::Fields::Unit) {
			return Err(syn::Error::new(Span::call_site(), "#[jenum] is only supported on c-like enums"));
		}

		let variant_id = v.ident.clone();
		let jvariant_id = convert_case::ccase!(upper_snake, variant_id.to_string());
		let jordinal = o as i32;

		get_java_variant.append_all(quote::quote!(
			Self::#variant_id => #jvariant_id,
		));

		get_rust_variant.append_all(quote::quote!(
			#jordinal => Ok(Self::#variant_id),
		));
	}

	let enum_ident = e.ident;
	let clazz = format!("{}/{}", attrs.package, attrs.clazz);

	Ok(quote::quote! {
		#original_enum

		impl<'local> jni_toolbox::IntoJavaObject<'local> for #enum_ident {
			const CLASS: &'static str = #clazz;
			fn into_java_object(
				self,
				env: &mut jni::Env<'local>,
			) -> Result<jni::objects::JObject<'local>, jni::errors::Error> {
				let ___clazz = env.find_class(jni::strings::JNIString::new(Self::CLASS))?;
				let ___variant_name = jni::strings::JNIString::new(match self {
					#get_java_variant
				});

				let sig = <Self as jni_toolbox::IntoJava<'local>>::signature();
				let sig_jni = jni::strings::JNIString::new(sig.0);
				let ___sig = unsafe {
					jni::signature::FieldSignature::from_raw_parts(
						sig_jni.borrowed(),
						sig.1
					)
				};

				env.get_static_field(___clazz, ___variant_name, ___sig)?.l()
			}
		}

		impl<'local> jni_toolbox::FromJavaObject<'local> for #enum_ident {
			fn from_java_object(
				object: jni::objects::JObject<'local>,
				env: &mut jni::Env<'local>,
			) -> Result<Self, jni::errors::Error> {
				let ordinal = env.call_method(object, jni::jni_str!("ordinal"), jni::jni_sig!("()I"), &[])?.i()?;
				match ordinal {
					#get_rust_variant
					_ => Err(jni::errors::Error::IndexOutOfBounds)
				}
			}
		}
	})
}


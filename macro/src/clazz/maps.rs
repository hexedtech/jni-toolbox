use proc_macro2::TokenStream;


pub fn sig(ty: syn::Type) -> TokenStream {
	match ty {
		syn::Type::Array(type_array) => {
			let inner = sig(*type_array.elem);
			quote::quote! { #inner[] }
		},
		syn::Type::Slice(type_slice) => {
			let inner = sig(*type_slice.elem);
			quote::quote! { #inner[] }
		},
		syn::Type::Path(type_path) => {
			if let Some(p) = type_path.path.segments.last().map(|x| x.ident.clone()) {
				return match p.to_string().as_str() {
					"String" => quote::quote! ( java.lang.String ),
					"bool" => quote::quote!( jboolean ),
					"i8" => quote::quote!( jbyte ),
					"i16" => quote::quote!( jshort ),
					"i32" => quote::quote!( jint ),
					"i64" => quote::quote!( jlong ),
					"f32" => quote::quote!( jfloat ),
					"f64" => quote::quote!( jdouble ),

					_ => panic!("unsuppoerted type: {p}"),
				}
			}
			panic!("wtfff");
		},
		// syn::Type::Ptr(type_ptr) => todo!(),
		// syn::Type::Reference(type_reference) => todo!(),
		// syn::Type::TraitObject(type_trait_object) => todo!(),
		// syn::Type::Tuple(type_tuple) => todo!(),
		// syn::Type::Verbatim(token_stream) => todo!(),
		// syn::Type::BareFn(type_bare_fn) => todo!(),
		// syn::Type::Group(type_group) => todo!(),
		// syn::Type::ImplTrait(type_impl_trait) => todo!(),
		// syn::Type::Infer(type_infer) => todo!(),
		// syn::Type::Macro(type_macro) => todo!(),
		// syn::Type::Never(type_never) => todo!(),
		// syn::Type::Paren(type_paren) => todo!(),
		_ => panic!("wtf"),
	}
}



fn is_primitive(ty: syn::Type) -> bool {
	match ty {
		syn::Type::Array(_) => false,
		syn::Type::BareFn(_) => false,
		syn::Type::Group(_) => false,
		syn::Type::ImplTrait(_) => false,
		syn::Type::Infer(type_infer) => todo!(),
		syn::Type::Macro(type_macro) => todo!(),
		syn::Type::Never(type_never) => todo!(),
		syn::Type::Paren(type_paren) => todo!(),
		syn::Type::Path(type_path) => todo!(),
		syn::Type::Ptr(_) => true,
		syn::Type::Reference(type_reference) => todo!(),
		syn::Type::Slice(type_slice) => todo!(),
		syn::Type::TraitObject(type_trait_object) => todo!(),
		syn::Type::Tuple(type_tuple) => todo!(),
		syn::Type::Verbatim(token_stream) => todo!(),
		_ => todo!(),
	}
}

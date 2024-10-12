
pub(crate) fn bare_type(ty: Box<syn::Type>) -> Option<syn::TypePath> {
	match *ty {
		syn::Type::Array(a) => bare_type(a.elem),
		syn::Type::BareFn(_) => None,
		syn::Type::ImplTrait(_) => None,
		syn::Type::Infer(_) => None,
		syn::Type::Macro(_) => None,
		syn::Type::Never(_) => None,
		syn::Type::TraitObject(_) => None,
		syn::Type::Verbatim(_) => None,
		syn::Type::Ptr(p) => bare_type(p.elem),
		syn::Type::Slice(s) => bare_type(s.elem),
		syn::Type::Tuple(t) => bare_type(Box::new(t.elems.first()?.clone())), // TODO
		syn::Type::Group(g) => bare_type(g.elem),
		syn::Type::Paren(p) => bare_type(p.elem),
		syn::Type::Reference(r) => bare_type(r.elem),
		syn::Type::Path(ty) => Some(ty),
		_ => todo!(),
	}
}

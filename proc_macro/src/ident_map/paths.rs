use super::*;

impl CollectIdents for Path {
	fn collect_idents(&self) {
		let Self {
			leading_colon: _,
			segments,
		} = self;
		collect!(segments);

		if let Some(seg) = segments.last() {
			cache_ambiguous(&seg.ident);
		}
	}
}

impl CollectIdents for PathSegment {
	fn collect_idents(&self) {
		let Self {
			ident: _,
			arguments,
		} = self;
		collect!(arguments);
	}
}

impl CollectIdents for PathArguments {
	fn collect_idents(&self) {
		match_collect!(self => PathArguments { AngleBracketed, Parenthesized, .. });
	}
}

impl CollectIdents for AssocConst {
	fn collect_idents(&self) {
		let Self {
			ident,
			generics,
			eq_token: _,
			value,
		} = self;
		cache_constant(ident);
		collect!(generics, value);
	}
}

impl CollectIdents for AssocType {
	fn collect_idents(&self) {
		let Self {
			ident: _,
			generics,
			eq_token: _,
			ty,
		} = self;
		collect!(generics, ty);
	}
}

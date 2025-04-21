use super::*;

impl CollectIdents for PatType {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			pat,
			colon_token: _,
			ty,
		} = self;
		collect!(pat, ty);
	}
}

impl CollectIdents for Pat {
	fn collect_idents(&self) {
		match_collect!(self => Pat {
			Const,
			Ident,
			Lit,
			Macro,
			Or,
			Paren,
			Path,
			Range,
			Reference,
			Rest,
			Slice,
			Struct,
			Tuple,
			TupleStruct,
			Type,
			Wild,
			..panic
		});
	}
}

impl CollectIdents for PatIdent {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			by_ref: _,
			mutability: _,
			ident: _,
			subpat,
		} = self;

		if let Some((_, pat)) = subpat {
			collect!(pat);
		}
	}
}

impl CollectIdents for PatOr {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			leading_vert: _,
			cases,
		} = self;
		collect!(cases);
	}
}

impl CollectIdents for PatParen {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			paren_token: _,
			pat,
		} = self;
		collect!(pat);
	}
}

impl CollectIdents for PatReference {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			and_token: _,
			mutability: _,
			pat,
		} = self;
		collect!(pat);
	}
}

impl CollectIdents for PatRest {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			dot2_token: _,
		} = self;
	}
}

impl CollectIdents for PatSlice {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			bracket_token: _,
			elems,
		} = self;
		collect!(elems);
	}
}

impl CollectIdents for PatStruct {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			qself,
			path,
			brace_token: _,
			fields,
			rest,
		} = self;
		collect!(qself, path, fields, rest);

		if let Some(QSelf {
			position: pos @ 1..,
			as_token: Some(..),
			..
		}) = &qself
		{
			let trait_segment = &path.segments[pos - 1];
			cache_trait(&trait_segment.ident);
		}

		if let Some(seg) = path.segments.last() {
			cache_ty(&seg.ident);
		}
	}
}

impl CollectIdents for FieldPat {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			member,
			colon_token: _,
			pat,
		} = self;
		collect!(member, pat);
	}
}

impl CollectIdents for PatTuple {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			paren_token: _,
			elems,
		} = self;
		collect!(elems);
	}
}

impl CollectIdents for PatTupleStruct {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			qself,
			path,
			paren_token: _,
			elems,
		} = self;
		collect!(qself, path, elems);

		if let Some(QSelf {
			position: pos @ 1..,
			as_token: Some(..),
			..
		}) = &qself
		{
			let trait_segment = &path.segments[pos - 1];
			cache_trait(&trait_segment.ident);
		}

		if let Some(seg) = path.segments.last() {
			cache_ty(&seg.ident);
		}
	}
}

impl CollectIdents for PatWild {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			underscore_token: _,
		} = self;
	}
}

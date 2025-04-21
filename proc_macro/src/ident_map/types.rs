use super::*;

impl CollectIdents for TypeArray {
	fn collect_idents(&self) {
		let Self {
			bracket_token: _,
			elem,
			semi_token: _,
			len,
		} = self;
		collect!(elem, len);
	}
}

impl CollectIdents for TypeSlice {
	fn collect_idents(&self) {
		let Self {
			bracket_token: _,
			elem,
		} = self;
		collect!(elem);
	}
}

impl CollectIdents for TypeGroup {
	fn collect_idents(&self) {
		let Self {
			group_token: _,
			elem,
		} = self;
		collect!(elem);
	}
}

impl CollectIdents for TypeParen {
	fn collect_idents(&self) {
		let Self {
			paren_token: _,
			elem,
		} = self;
		collect!(elem);
	}
}

impl CollectIdents for TypePtr {
	fn collect_idents(&self) {
		let Self {
			star_token: _,
			const_token: _,
			mutability: _,
			elem,
		} = self;
		collect!(elem);
	}
}

impl CollectIdents for TypeBareFn {
	fn collect_idents(&self) {
		let Self {
			lifetimes,
			unsafety: _,
			abi: _,
			fn_token: _,
			paren_token: _,
			inputs,
			variadic: _,
			output,
		} = self;
		collect!(lifetimes, inputs, output);
	}
}

impl CollectIdents for BareFnArg {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			name: _,
			ty,
		} = self;
		collect!(ty);
	}
}

impl CollectIdents for TypeImplTrait {
	fn collect_idents(&self) {
		let Self {
			impl_token: _,
			bounds,
		} = self;
		collect!(bounds);
	}
}

impl CollectIdents for TypeTraitObject {
	fn collect_idents(&self) {
		let Self {
			dyn_token: _,
			bounds,
		} = self;
		collect!(bounds);
	}
}

impl CollectIdents for TypeTuple {
	fn collect_idents(&self) {
		let Self {
			paren_token: _,
			elems,
		} = self;
		collect!(elems);
	}
}

impl CollectIdents for TypeReference {
	fn collect_idents(&self) {
		let Self {
			and_token: _,
			lifetime,
			mutability: _,
			elem,
		} = self;
		collect!(lifetime, elem);
	}
}

impl CollectIdents for TypeInfer {
	fn collect_idents(&self) {
		let Self {
			underscore_token: _,
		} = self;
	}
}

impl CollectIdents for TypeMacro {
	fn collect_idents(&self) { let Self { mac: _ } = self; }
}

impl CollectIdents for TypeNever {
	fn collect_idents(&self) { let Self { bang_token: _ } = self; }
}

impl CollectIdents for Type {
	fn collect_idents(&self) {
		match_collect!(self => Type {
			Array,
			Slice,
			Group,
			Paren,
			Ptr,
			Path,
			BareFn,
			ImplTrait,
			Reference,
			TraitObject,
			Tuple,
			Infer,
			Macro,
			Never,
			..panic
		})
	}
}

impl CollectIdents for QSelf {
	fn collect_idents(&self) {
		let Self {
			lt_token: _,
			ty,
			position: _,
			as_token: _,
			gt_token: _,
		} = self;
		collect!(ty);
	}
}

impl CollectIdents for TypePath {
	fn collect_idents(&self) {
		let Self { qself, path } = self;
		collect!(qself, path);

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

impl CollectIdents for ReturnType {
	fn collect_idents(&self) {
		if let ReturnType::Type(_, ty) = self {
			collect!(ty);
		}
	}
}

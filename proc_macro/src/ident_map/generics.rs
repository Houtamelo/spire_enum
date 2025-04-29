use super::*;

impl CollectIdents for Generics {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			lt_token: _,
			params,
			gt_token: _,
			where_clause,
		} = self;
		collect!(map, params, where_clause);
	}
}

impl CollectIdents for WhereClause {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			where_token: _,
			predicates,
		} = self;
		collect!(map, predicates);
	}
}

impl CollectIdents for WherePredicate {
	fn collect_idents(&self, map: &mut IdentMap) {
		match_collect!(map, self => WherePredicate{Lifetime, Type, ..panic });
	}
}

impl CollectIdents for PredicateLifetime {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			lifetime,
			colon_token: _,
			bounds,
		} = self;
		collect!(map, lifetime, bounds);
	}
}

impl CollectIdents for PredicateType {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			lifetimes,
			bounded_ty,
			colon_token: _,
			bounds,
		} = self;
		collect!(map, lifetimes, bounded_ty, bounds);
	}
}

impl CollectIdents for GenericParam {
	fn collect_idents(&self, map: &mut IdentMap) {
		match_collect!(map, self => GenericParam{Lifetime, Type, Const});
	}
}

impl CollectIdents for ConstParam {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			attrs: _,
			const_token: _,
			ident,
			colon_token: _,
			ty,
			eq_token: _,
			default: _,
		} = self;
		map.insert_constant(ident);
		collect!(map, ty);
	}
}

impl CollectIdents for LifetimeParam {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			attrs: _,
			lifetime,
			colon_token: _,
			bounds,
		} = self;
		collect!(map, lifetime, bounds);
	}
}

impl CollectIdents for TypeParam {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			attrs: _,
			ident,
			colon_token: _,
			bounds,
			eq_token: _,
			default,
		} = self;
		map.insert_ty(ident);
		collect!(map, bounds, default);
	}
}

impl CollectIdents for TraitBound {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			paren_token: _,
			modifier: _,
			lifetimes,
			path,
		} = self;
		collect!(map, lifetimes, path);

		if let Some(seg) = path.segments.last() {
			map.insert_trait(&seg.ident);
		}
	}
}

impl CollectIdents for TypeParamBound {
	fn collect_idents(&self, map: &mut IdentMap) {
		match_collect!(map, self => TypeParamBound{ Trait, Lifetime, PreciseCapture, ..panic });
	}
}

impl CollectIdents for BoundLifetimes {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			for_token: _,
			lt_token: _,
			lifetimes,
			gt_token: _,
		} = self;
		collect!(map, lifetimes);
	}
}

impl CollectIdents for GenericArgument {
	fn collect_idents(&self, map: &mut IdentMap) {
		match_collect!(map, self => GenericArgument {
			Lifetime,
			Constraint,
			Type,
			AssocType,
			Const,
			AssocConst,
			..panic
		});
	}
}

impl CollectIdents for Constraint {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			ident,
			generics,
			colon_token: _,
			bounds,
		} = self;
		map.insert_trait(ident);
		collect!(map, generics, bounds);
	}
}

impl CollectIdents for AngleBracketedGenericArguments {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			colon2_token: _,
			lt_token: _,
			args,
			gt_token: _,
		} = self;
		collect!(map, args);
	}
}

impl CollectIdents for ParenthesizedGenericArguments {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			paren_token: _,
			inputs,
			output,
		} = self;
		collect!(map, inputs, output);
	}
}

impl CollectIdents for PreciseCapture {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			use_token: _,
			lt_token: _,
			params,
			gt_token: _,
		} = self;
		collect!(map, params);
	}
}

impl CollectIdents for CapturedParam {
	fn collect_idents(&self, map: &mut IdentMap) {
		match self {
			CapturedParam::Lifetime(life) => collect!(map, life),
			CapturedParam::Ident(ident) => map.insert_ty(ident),
			_ => panic!("Unsupported variant: {self:?}"),
		}
	}
}

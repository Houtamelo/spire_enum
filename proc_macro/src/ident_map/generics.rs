use super::*;

impl CollectIdents for Generics {
	fn collect_idents(&self) {
		let Self {
			lt_token: _,
			params,
			gt_token: _,
			where_clause,
		} = self;
		collect!(params, where_clause);
	}
}

impl CollectIdents for SaneGenerics {
	fn collect_idents(&self) {
		let Self {
			input: stage0,
			where_clause,
		} = self;
		collect!(stage0, where_clause);
	}
}

impl CollectIdents for InputGenerics {
	fn collect_idents(&self) {
		let Self {
			_left_angle_bracket,
			params,
			_right_angle_bracket,
		} = self;
		collect!(params);
	}
}

impl CollectIdents for WhereClause {
	fn collect_idents(&self) {
		let Self {
			where_token: _,
			predicates,
		} = self;
		collect!(predicates);
	}
}

impl CollectIdents for WherePredicate {
	fn collect_idents(&self) {
		match_collect!(self => WherePredicate{Lifetime, Type, ..panic });
	}
}

impl CollectIdents for PredicateLifetime {
	fn collect_idents(&self) {
		let Self {
			lifetime,
			colon_token: _,
			bounds,
		} = self;
		collect!(lifetime, bounds);
	}
}

impl CollectIdents for PredicateType {
	fn collect_idents(&self) {
		let Self {
			lifetimes,
			bounded_ty,
			colon_token: _,
			bounds,
		} = self;
		collect!(lifetimes, bounded_ty, bounds);
	}
}

impl CollectIdents for GenericParam {
	fn collect_idents(&self) {
		match_collect!(self => GenericParam{Lifetime, Type, Const});
	}
}

impl CollectIdents for InputGenericParam {
	fn collect_idents(&self) {
		match_collect!(self => InputGenericParam{Lifetime, Type, Const});
	}
}

impl CollectIdents for ConstParam {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			const_token: _,
			ident,
			colon_token: _,
			ty,
			eq_token: _,
			default: _,
		} = self;
		cache_constant(ident);
		collect!(ty);
	}
}

impl CollectIdents for InputGenericParamConst {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			const_token: _,
			ident,
			colon_token: _,
			ty,
			eq_token: _,
			default: _,
		} = self;
		cache_constant(ident);
		collect!(ty);
	}
}

impl CollectIdents for LifetimeParam {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			lifetime,
			colon_token: _,
			bounds,
		} = self;
		collect!(lifetime, bounds);
	}
}

impl CollectIdents for InputGenericParamLifetime {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			lifetime,
			bounds,
		} = self;

		collect!(lifetime);
		match bounds {
			InputGenericParamLifetimeBounds::Some(_, list) => collect!(list),
			InputGenericParamLifetimeBounds::None => {}
		}
	}
}

impl CollectIdents for TypeParam {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			ident,
			colon_token: _,
			bounds,
			eq_token: _,
			default,
		} = self;
		cache_ty(ident);
		collect!(bounds, default);
	}
}

impl CollectIdents for InputGenericParamType {
	fn collect_idents(&self) {
		let Self {
			attrs: _,
			ident,
			colon_token: _,
			bounds,
			eq_token: _,
			default,
		} = self;
		cache_ty(ident);
		collect!(bounds, default);
	}
}

impl CollectIdents for TraitBound {
	fn collect_idents(&self) {
		let Self {
			paren_token: _,
			modifier: _,
			lifetimes,
			path,
		} = self;
		collect!(lifetimes, path);

		if let Some(seg) = path.segments.last() {
			cache_trait(&seg.ident);
		}
	}
}

impl CollectIdents for TypeParamBound {
	fn collect_idents(&self) {
		match_collect!(self => TypeParamBound{ Trait, Lifetime, PreciseCapture, ..panic });
	}
}

impl CollectIdents for BoundLifetimes {
	fn collect_idents(&self) {
		let Self {
			for_token: _,
			lt_token: _,
			lifetimes,
			gt_token: _,
		} = self;
		collect!(lifetimes);
	}
}

impl CollectIdents for GenericArgument {
	fn collect_idents(&self) {
		match_collect!(self => GenericArgument {
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
	fn collect_idents(&self) {
		let Self {
			ident,
			generics,
			colon_token: _,
			bounds,
		} = self;
		cache_trait(ident);
		collect!(generics, bounds);
	}
}

impl CollectIdents for AngleBracketedGenericArguments {
	fn collect_idents(&self) {
		let Self {
			colon2_token: _,
			lt_token: _,
			args,
			gt_token: _,
		} = self;
		collect!(args);
	}
}

impl CollectIdents for ParenthesizedGenericArguments {
	fn collect_idents(&self) {
		let Self {
			paren_token: _,
			inputs,
			output,
		} = self;
		collect!(inputs, output);
	}
}

impl CollectIdents for PreciseCapture {
	fn collect_idents(&self) {
		let Self {
			use_token: _,
			lt_token: _,
			params,
			gt_token: _,
		} = self;
		collect!(params);
	}
}

impl CollectIdents for CapturedParam {
	fn collect_idents(&self) {
		match self {
			CapturedParam::Lifetime(life) => collect!(life),
			CapturedParam::Ident(ident) => cache_ty(ident),
			_ => panic!("Unsupported variant: {self:?}"),
		}
	}
}

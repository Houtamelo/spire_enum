use super::*;

#[derive(Parse, ToTokens)]
pub struct Var<Attr, FieldsAttr> {
	pub attrs: Any<Attribute<Attr>>,
	pub ident: Ident,
	pub fields: VarFields<FieldsAttr>,
	pub discriminant: Optional<InputDiscriminant>,
}

#[derive(Parse, ToTokens)]
pub enum VarFields<Attr> {
	Named(Brace<Punctuated<VarFieldNamed<Attr>, Token![,]>>),
	Unnamed(Paren<Punctuated<VarFieldUnnamed<Attr>, Token![,]>>),
	Unit,
}

impl<T> VarFields<T> {
	pub fn len(&self) -> usize {
		match self {
			VarFields::Named(fields) => fields.len(),
			VarFields::Unnamed(fields) => fields.len(),
			VarFields::Unit => 0,
		}
	}
}

#[derive(Parse, ToTokens)]
pub struct VarFieldNamed<Attr> {
	pub attrs: Any<Attribute<Attr>>,
	pub ident: Ident,
	pub colon_token: Option<Token![:]>,
	pub ty: Type,
}

#[derive(Parse, ToTokens)]
pub struct VarFieldUnnamed<Attr> {
	pub attrs: Any<Attribute<Attr>>,
	pub ty: Type,
}

#[derive(Clone, Parse, ToTokens)]
pub struct InputDiscriminant {
	pub eq_token: Token![=],
	pub expr: Expr,
}

pub fn generics_needed_by_variant<T>(
	fields: &VarFields<T>,
	enum_generics: &Optional<SaneGenerics>,
) -> Optional<SaneGenerics> {
	let _Some(enum_generics) = enum_generics.as_ref() else { return _None };

	let (enum_lfs, enum_tys, enum_consts) = {
		let mut tys = Vec::new();
		let mut lfs = Vec::new();
		let mut consts = Vec::new();

		for param in &enum_generics.input.params {
			match param {
				GenericParam::Lifetime(lifetime) => lfs.push(lifetime),
				GenericParam::Type(ty) => tys.push(ty),
				GenericParam::Const(constant) => consts.push(constant),
			}
		}

		(lfs, tys, consts)
	};

	macro_rules! has_ty {
		($Collection:expr, $item:expr) => {
			$Collection.iter().any(|t| t.ident == $item)
		};
	}

	macro_rules! has_const {
		($Collection:expr, $item:expr) => {
			$Collection.iter().any(|t| t.ident == $item)
		};
	}

	macro_rules! has_lf {
		($Collection:expr, $item:expr) => {
			$Collection.iter().any(|t| t.lifetime.ident == $item)
		};
	}

	macro_rules! has {
		($Collection:expr, $F:expr) => {
			$Collection.iter().any(|item| $F == item)
		};
	}

	let var_idents = IdentMap::new(fields);

	let var_lfs = enum_lfs
		.iter()
		.cloned()
		.filter(|param| has!(var_idents.lifetimes, param.lifetime.ident))
		.collect::<Vec<_>>();

	let var_tys = enum_tys
		.iter()
		.cloned()
		.filter(|param| {
			has!(var_idents.tys, param.ident) || has!(var_idents.ambiguous_paths, param.ident)
		})
		.collect::<Vec<_>>();

	let var_consts = enum_consts
		.iter()
		.cloned()
		.filter(|param| {
			has!(var_idents.constants, param.ident) || has!(var_idents.ambiguous_paths, param.ident)
		})
		.collect::<Vec<_>>();

	let len = var_tys.len() + var_lfs.len() + var_consts.len();
	if len == 0 {
		return _None;
	}

	macro_rules! validate_idents {
		($Map:ident) => {
			'ret: {
				for item in &$Map.lifetimes {
					let is_generic = has_lf!(enum_lfs, item);
					let var_needs = has_lf!(var_lfs, item);

					if is_generic && !var_needs {
						break 'ret false;
					}
				}

				for item in &$Map.tys {
					let is_generic = has_ty!(enum_tys, item);
					let var_needs = has_ty!(var_tys, item);

					if is_generic && !var_needs {
						break 'ret false;
					}
				}

				for item in &$Map.constants {
					let is_generic = has_const!(enum_consts, item);
					let var_needs = has_const!(var_consts, item);

					if is_generic && !var_needs {
						break 'ret false;
					}
				}

				true
			}
		};
	}

	let mut params = Punctuated::<GenericParam, Token![,]>::new();

	params.extend(var_lfs.iter().map(|&lf_param| {
		let bounds = lf_param
			.bounds
			.iter()
			.filter(|l| has_lf!(var_lfs, l.ident))
			.cloned()
			.collect::<syn::punctuated::Punctuated<Lifetime, Token![+]>>();

		let colon_token = if !bounds.is_empty() {
			Some(lf_param.colon_token.unwrap_or_default())
		} else {
			None
		};

		GenericParam::Lifetime(LifetimeParam {
			colon_token,
			bounds,
			..lf_param.clone()
		})
	}));

	params.extend(var_tys.iter().map(|&ty_param| {
		let bounds = ty_param
			.bounds
			.iter()
			.filter(|bound| {
				let bound_idents = IdentMap::new(*bound);
				validate_idents!(bound_idents)
			})
			.cloned()
			.collect::<syn::punctuated::Punctuated<TypeParamBound, Token![+]>>();

		let colon_token = if !bounds.is_empty() {
			Some(ty_param.colon_token.unwrap_or_default())
		} else {
			None
		};

		GenericParam::Type(TypeParam {
			colon_token,
			bounds,
			..ty_param.clone()
		})
	}));

	params.extend(var_consts.iter().map(|&c| GenericParam::Const(c.clone())));

	let where_clause = match &enum_generics.where_clause {
		_Some(wc) => {
			let predicates = wc
				.predicates
				.iter()
				.filter(|wc| {
					let wc_idents = IdentMap::new(*wc);
					validate_idents!(wc_idents)
				})
				.cloned()
				.collect::<syn::punctuated::Punctuated<WherePredicate, Token![,]>>();

			if !predicates.is_empty() {
				_Some(WhereClause {
					predicates,
					..wc.clone()
				})
			} else {
				_None
			}
		}
		_None => _None,
	};

	_Some(SaneGenerics {
		input: InputGenerics {
			lb_token: enum_generics.input.lb_token,
			params:   params.into(),
			rb_token: enum_generics.input.rb_token,
		},
		where_clause,
	})
}

impl<T> CollectIdents for VarFields<T> {
	fn collect_idents(&self, map: &mut IdentMap) {
		match_collect!(map, self => VarFields { Named, Unnamed, .. });
	}
}

impl<T> CollectIdents for VarFieldNamed<T> {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			attrs: _,
			ident: _,
			colon_token: _,
			ty,
		} = self;
		collect!(map, ty);
	}
}

impl<T> CollectIdents for VarFieldUnnamed<T> {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self { attrs: _, ty } = self;
		collect!(map, ty);
	}
}

use super::*;

pub struct SaneVariant {
	pub attrs: SaneVariantAttributes,
	pub ident: Ident,
	pub fields: SaneVariantFields,
	pub discriminant: Optional<InputVariantDiscriminant>,
	pub generics: Optional<SaneGenerics>,
	pub explicit_delegator: Optional<ExplicitDelegator>,
}

impl SaneVariant {
	pub fn allow_generate_type(&self) -> bool { self.attrs.no_var_type.is_none() }
	pub fn allow_generate_conversions(&self) -> bool { self.attrs.no_convert.is_none() }
}

pub fn sanitize_variant(
	variant: InputVariant,
	settings: &Settings,
	enum_generics: &Optional<SaneGenerics>,
) -> Result<SaneVariant> {
	let attrs = sanitize_attributes(variant.attrs)?;
	let explicit_delegator = find_delegator(&attrs)?;

	if let _Some(ExplicitDelegator::Expr(_, expr)) = &explicit_delegator {
		if settings.generate_variants.is_some()
			&& attrs.no_var_type.is_none()
			&& expr.inputs.len() > 1
		{
			bail!(expr => "a type will be generated for this variant, the delegator closure should either \
					take no parameters or take the variant's type as the single parameter");
		}

		if expr.inputs.len() > variant.fields.field_count() {
			bail!(expr => "delegator closure should not take more parameters than variant's field count");
		}
	}

	let fields = sanitize_variant_fields(variant.fields)?;
	if let (_Some(a), Some(b)) = (&explicit_delegator, fields.delegator_field_kw()) {
		err_expected_only_one!(a, b)
	}

	let generics = generics_needed_by_variant(&fields, enum_generics);

	Ok(SaneVariant {
		attrs,
		ident: variant.ident,
		fields,
		discriminant: variant.discriminant,
		generics,
		explicit_delegator,
	})
}

#[derive(Default)]
pub struct SaneVariantAttributes {
	pub syn_attrs: Vec<Meta>,
	pub no_var_type: Option<var_kw::dont_generate_type>,
	pub no_convert: Option<var_kw::dont_impl_conversions>,
	pub delegate_via: Option<(var_kw::delegate_via, Paren<ExprClosure>)>,
}

fn sanitize_attributes(attrs: Any<InputAttribute<VariantMeta>>) -> Result<SaneVariantAttributes> {
	let mut sane = SaneVariantAttributes::default();

	for attr in attrs {
		match attr.inner.into_inner() {
			VariantMeta::NoVarType(kw) => assign_unique_or_panic!(sane.no_var_type, kw),
			VariantMeta::NoConversions(kw) => assign_unique_or_panic!(sane.no_convert, kw),
			VariantMeta::DelegateVia(kw, expr) => {
				if let Some((first_kw, _)) = sane.delegate_via {
					err_expected_only_one!(first_kw, kw);
				} else {
					sane.delegate_via = Some((kw, expr));
				}
			}
			VariantMeta::Syn(syn_attr) => sane.syn_attrs.push(syn_attr),
		}
	}

	Ok(sane)
}

#[derive(ToTokens)]
pub enum ExplicitDelegator {
	Expr(#[allow(unused)] var_kw::delegate_via, Box<Paren<ExprClosure>>),
}

fn find_delegator(attrs: &SaneVariantAttributes) -> Result<Optional<ExplicitDelegator>> {
	match attrs.delegate_via.clone() {
		Some((kw, expr)) => Ok(_Some(ExplicitDelegator::Expr(kw, expr.into()))),
		None => Ok(_None),
	}
}

fn generics_needed_by_variant(
	fields: &SaneVariantFields,
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
			_left_angle_bracket: enum_generics.input._left_angle_bracket,
			params: params.into(),
			_right_angle_bracket: enum_generics.input._right_angle_bracket,
		},
		where_clause,
	})
}

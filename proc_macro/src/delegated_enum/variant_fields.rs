pub use attr_kw::delegator as kw_delegator;

use super::*;

mod attr_kw {
	use super::*;
	custom_keyword!(delegator);
}

#[derive(ToTokens)]
pub enum SaneVarFields {
	Named(SaneVarFieldsNamed),
	Unnamed(SaneVarFieldsUnnamed),
	Unit,
}

impl SaneVarFields {
	pub fn delegator_field_kw(&self) -> Option<&kw_delegator> {
		match self {
			SaneVarFields::Named(named) => named.delegator.as_ref().map(|(kw, _)| kw),
			SaneVarFields::Unnamed(unnamed) => unnamed.delegator.as_ref().map(|(kw, _)| kw),
			SaneVarFields::Unit => None,
		}
	}
}

pub struct SaneVarFieldsNamed {
	pub fields: Brace<Punctuated<VarFieldNamed<SynMeta>, Token![,]>>,
	pub delegator: Option<(kw_delegator, Ident)>,
}

pub struct SaneVarFieldsUnnamed {
	pub fields: Paren<Punctuated<VarFieldUnnamed<SynMeta>, Token![,]>>,
	pub delegator: Option<(kw_delegator, usize)>,
}

pub fn sanitize_variant_fields(input: VarFields<Meta<kw_delegator>>) -> Result<SaneVarFields> {
	match input {
		VarFields::Named(fields) => {
			let (brace_token, fields) = fields.into_parts();

			let mut delegator: Option<(kw_delegator, Ident)> = None;
			let mut new_fields = Punctuated::<VarFieldNamed<SynMeta>, Token![,]>::new();

			for pair in fields.into_pairs() {
				let (input_field, comma) = pair.into_tuple();
				let (new_field, delegator_option) = sanitize_field_named(input_field)?;
				if let _Some(second) = delegator_option {
					if let Some((first, _)) = delegator {
						err_expected_only_one!(first, second);
					} else {
						delegator = Some((second, new_field.ident.clone()));
					}
				}

				new_fields.push_value(new_field);
				if let Some(comma) = comma {
					new_fields.push_punct(comma);
				}
			}

			Ok(SaneVarFields::Named(SaneVarFieldsNamed {
				fields: Brace::from((brace_token, new_fields)),
				delegator,
			}))
		}
		VarFields::Unnamed(fields) => {
			let (paren_token, fields) = fields.into_parts();

			let mut delegator: Option<(kw_delegator, usize)> = None;
			let mut new_fields = Punctuated::<VarFieldUnnamed<SynMeta>, Token![,]>::new();

			for (idx, pair) in fields.into_pairs().enumerate() {
				let (input_field, comma) = pair.into_tuple();
				let (new_field, delegator_option) = sanitize_field_unnamed(input_field)?;
				if let _Some(second) = delegator_option {
					if let Some((first, _)) = delegator {
						err_expected_only_one!(first, second);
					} else {
						delegator = Some((second, idx));
					}
				}

				new_fields.push_value(new_field);
				if let Some(comma) = comma {
					new_fields.push_punct(comma);
				}
			}

			Ok(SaneVarFields::Unnamed(SaneVarFieldsUnnamed {
				fields: Paren::from((paren_token, new_fields)),
				delegator,
			}))
		}
		VarFields::Unit => Ok(SaneVarFields::Unit),
	}
}

fn sanitize_field_named(
	field: VarFieldNamed<Meta<kw_delegator>>,
) -> Result<(VarFieldNamed<SynMeta>, Optional<kw_delegator>)> {
	let VarFieldNamed {
		attrs,
		ident,
		colon_token,
		ty,
	} = field;

	let (attrs, custom_metas) = split_input_attrs(attrs.into_inner());

	let mut delegator: Optional<kw_delegator> = _None;

	for attr in custom_metas {
		let kw = attr.inner.into_inner();
		assign_unique_or_panic!(delegator, kw);
	}

	Ok((
		VarFieldNamed {
			attrs,
			ident,
			colon_token,
			ty,
		},
		delegator,
	))
}

fn sanitize_field_unnamed(
	field: VarFieldUnnamed<Meta<kw_delegator>>,
) -> Result<(VarFieldUnnamed<SynMeta>, Optional<kw_delegator>)> {
	let VarFieldUnnamed { attrs, ty } = field;

	let (attrs, custom_metas) = split_input_attrs(attrs.into_inner());
	let mut delegator: Optional<kw_delegator> = _None;

	for attr in custom_metas {
		let kw = attr.inner.into_inner();
		assign_unique_or_panic!(delegator, kw);
	}

	Ok((VarFieldUnnamed { attrs, ty }, delegator))
}

impl ToTokens for SaneVarFieldsNamed {
	fn to_tokens(&self, tokens: &mut TokenStream) { self.fields.to_tokens(tokens); }
}

impl ToTokens for SaneVarFieldsUnnamed {
	fn to_tokens(&self, tokens: &mut TokenStream) { self.fields.to_tokens(tokens); }
}

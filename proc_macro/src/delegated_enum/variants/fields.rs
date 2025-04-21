use super::*;

mod attr_kw {
	use super::*;
	custom_keyword!(delegator);
}

#[derive(Parse, ToTokens)]
pub enum VariantFields<Attr> {
	Named(Brace<Punctuated<VariantFieldNamed<Attr>, Token![,]>>),
	Unnamed(Paren<Punctuated<VariantFieldUnnamed<Attr>, Token![,]>>),
	Unit,
}

impl<T> VariantFields<T> {
	pub fn field_count(&self) -> usize {
		match self {
			VariantFields::Named(fields) => fields.len(),
			VariantFields::Unnamed(fields) => fields.len(),
			VariantFields::Unit => 0,
		}
	}
}

#[derive(Parse, ToTokens)]
pub struct VariantFieldNamed<Attr> {
	pub attrs: Any<InputAttribute<Attr>>,
	pub ident: Ident,
	pub colon_token: Option<Token![:]>,
	pub ty: Type,
}

#[derive(Parse, ToTokens)]
pub struct VariantFieldUnnamed<Attr> {
	pub attrs: Any<InputAttribute<Attr>>,
	pub ty: Type,
}

#[allow(clippy::large_enum_variant)]
#[derive(Parse, ToTokens)]
pub enum VariantFieldMeta {
	Delegator(attr_kw::delegator),
	Syn(Meta),
}

#[derive(ToTokens)]
pub enum SaneVariantFields {
	Named(SaneVariantFieldsNamed),
	Unnamed(SaneVariantFieldsUnnamed),
	Unit,
}

impl SaneVariantFields {
	pub fn delegator_field_kw(&self) -> Option<&attr_kw::delegator> {
		match self {
			SaneVariantFields::Named(named) => named.delegator.as_ref().map(|(kw, _)| kw),
			SaneVariantFields::Unnamed(unnamed) => unnamed.delegator.as_ref().map(|(kw, _)| kw),
			SaneVariantFields::Unit => None,
		}
	}
}

pub struct SaneVariantFieldsNamed {
	pub fields: Brace<Punctuated<VariantFieldNamed<Meta>, Token![,]>>,
	pub delegator: Option<(attr_kw::delegator, Ident)>,
}

pub struct SaneVariantFieldsUnnamed {
	pub fields: Paren<Punctuated<VariantFieldUnnamed<Meta>, Token![,]>>,
	pub delegator: Option<(attr_kw::delegator, usize)>,
}

pub fn sanitize_variant_fields(
	input: VariantFields<VariantFieldMeta>,
) -> Result<SaneVariantFields> {
	match input {
		VariantFields::Named(fields) => {
			let (brace_token, fields) = fields.into_parts();

			let mut delegator: Option<(attr_kw::delegator, Ident)> = None;
			let mut new_fields = Punctuated::<VariantFieldNamed<Meta>, Token![,]>::new();

			for pair in fields.into_pairs() {
				let (input_field, comma) = pair.into_tuple();
				let (new_field, delegator_option) = sanitize_field_named(input_field)?;
				if let Some(second) = delegator_option {
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

			Ok(SaneVariantFields::Named(SaneVariantFieldsNamed {
				fields: Brace::from((brace_token, new_fields)),
				delegator,
			}))
		}
		VariantFields::Unnamed(fields) => {
			let (paren_token, fields) = fields.into_parts();

			let mut delegator: Option<(attr_kw::delegator, usize)> = None;
			let mut new_fields = Punctuated::<VariantFieldUnnamed<Meta>, Token![,]>::new();

			for (idx, pair) in fields.into_pairs().enumerate() {
				let (input_field, comma) = pair.into_tuple();
				let (new_field, delegator_option) = sanitize_field_unnamed(input_field)?;
				if let Some(second) = delegator_option {
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

			Ok(SaneVariantFields::Unnamed(SaneVariantFieldsUnnamed {
				fields: Paren::from((paren_token, new_fields)),
				delegator,
			}))
		}
		VariantFields::Unit => Ok(SaneVariantFields::Unit),
	}
}

fn sanitize_field_named(
	field: VariantFieldNamed<VariantFieldMeta>,
) -> Result<(VariantFieldNamed<Meta>, Option<attr_kw::delegator>)> {
	let VariantFieldNamed {
		attrs,
		ident,
		colon_token,
		ty,
	} = field;

	let mut delegator: Option<attr_kw::delegator> = None;

	let mut new_attrs = Any::<InputAttribute<Meta>>::new();
	for InputAttribute { pound_token, inner } in attrs {
		let (bracket, meta) = inner.into_parts();

		match meta {
			VariantFieldMeta::Delegator(kw) => {
				assign_unique_or_panic!(delegator, kw);
			}
			VariantFieldMeta::Syn(meta) => {
				let new_attr = InputAttribute {
					pound_token,
					inner: Bracket::from((bracket, meta)),
				};
				new_attrs.push(new_attr);
			}
		}
	}

	let new_field = VariantFieldNamed {
		attrs: new_attrs,
		ident,
		colon_token,
		ty,
	};

	Ok((new_field, delegator))
}

fn sanitize_field_unnamed(
	field: VariantFieldUnnamed<VariantFieldMeta>,
) -> Result<(VariantFieldUnnamed<Meta>, Option<attr_kw::delegator>)> {
	let VariantFieldUnnamed { attrs, ty } = field;

	let mut delegator: Option<attr_kw::delegator> = None;

	let mut new_attrs = Any::<InputAttribute<Meta>>::new();
	for InputAttribute { pound_token, inner } in attrs {
		let (bracket, meta) = inner.into_parts();

		match meta {
			VariantFieldMeta::Delegator(kw) => {
				assign_unique_or_panic!(delegator, kw);
			}
			VariantFieldMeta::Syn(meta) => {
				let new_attr = InputAttribute {
					pound_token,
					inner: Bracket::from((bracket, meta)),
				};
				new_attrs.push(new_attr);
			}
		}
	}

	let new_field = VariantFieldUnnamed {
		attrs: new_attrs,
		ty,
	};

	Ok((new_field, delegator))
}

impl ToTokens for SaneVariantFieldsNamed {
	fn to_tokens(&self, tokens: &mut TokenStream) { self.fields.to_tokens(tokens); }
}

impl ToTokens for SaneVariantFieldsUnnamed {
	fn to_tokens(&self, tokens: &mut TokenStream) { self.fields.to_tokens(tokens); }
}

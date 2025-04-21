use super::*;

mod kw {
	use super::*;
	custom_keyword!(derive);
	custom_keyword!(attrs);
	custom_keyword!(generate_variants);
	custom_keyword!(impl_enum_try_into_variants);
	custom_keyword!(impl_variants_into_enum);
	custom_keyword!(impl_conversions);
}

pub struct Settings {
	pub generate_variants: Option<SaneGenerateVariants>,
	enum_try_into_variants: Option<kw::impl_enum_try_into_variants>,
	variants_into_enum: Option<kw::impl_variants_into_enum>,
	conversions: Option<kw::impl_conversions>,
}

impl Settings {
	pub fn should_impl_enum_try_into_variants(&self) -> bool {
		self.enum_try_into_variants.is_some() || self.conversions.is_some()
	}

	pub fn should_impl_variants_into_enum(&self) -> bool {
		self.variants_into_enum.is_some() || self.conversions.is_some()
	}
}

pub fn parse_settings(input_stream: TokenStream1) -> Result<Settings> {
	let setting_list = syn::parse::<Punctuated<Setting, Token![,]>>(input_stream)?;

	let mut generate_variants: Option<SaneGenerateVariants> = None;
	let mut enum_try_into_variants: Option<kw::impl_enum_try_into_variants> = None;
	let mut variants_into_enum: Option<kw::impl_variants_into_enum> = None;
	let mut conversions: Option<kw::impl_conversions> = None;

	for setting in setting_list {
		match setting {
			Setting::GenerateVariants(kw, paren_option) => {
				if let Some(first_gen_vars) = generate_variants {
					err_expected_only_one!(first_gen_vars.kw, kw);
				} else {
					generate_variants = Some(sanitize_generate_variant_types(kw, paren_option)?);
				}
			}
			Setting::ImplVariantsTryFromEnum(kw) => {
				assign_unique_or_panic!(enum_try_into_variants, kw)
			}
			Setting::ImplEnumFromVars(kw) => {
				assign_unique_or_panic!(variants_into_enum, kw)
			}
			Setting::ImplConversions(kw) => {
				assign_unique_or_panic!(conversions, kw)
			}
		}
	}

	match (enum_try_into_variants, variants_into_enum, conversions) {
		| (_, Some(..), Some(..)) | (Some(..), _, Some(..)) => {
			bail!(conversions => "Setting `conversions` already implies `enum_try_into_variants` and `variants_into_enum`")
		}
		_ => {}
	}

	Ok(Settings {
		generate_variants,
		enum_try_into_variants,
		variants_into_enum,
		conversions,
	})
}

#[derive(Parse, ToTokens)]
enum Setting {
	GenerateVariants(
		kw::generate_variants,
		Optional<Paren<Punctuated<GenerateVariantTypesSetting, Token![,]>>>,
	),
	ImplVariantsTryFromEnum(kw::impl_enum_try_into_variants),
	ImplEnumFromVars(kw::impl_variants_into_enum),
	ImplConversions(kw::impl_conversions),
}

#[derive(Parse, ToTokens)]
enum GenerateVariantTypesSetting {
	Attrs(SettingAttrs),
	Derive(SettingDerive),
}

/// Shorthand for `each_attributes = [derive(...)]`
#[derive(Parse, ToTokens)]
struct SettingDerive {
	pub kw: kw::derive,
	pub paths: Paren<Punctuated<Path, Token![,]>>,
}

#[derive(Parse, ToTokens)]
struct SettingAttrs {
	pub kw: kw::attrs,
	pub eq_token: Token![=],
	pub attrs: Bracket<Punctuated<Meta, Token![,]>>,
}

#[derive(Default)]
pub struct SaneGenerateVariants {
	pub kw: kw::generate_variants,
	pub attrs: Vec<Meta>,
}

fn sanitize_generate_variant_types(
	kw: kw::generate_variants,
	paren_option: Optional<Paren<Punctuated<GenerateVariantTypesSetting, Token![,]>>>,
) -> Result<SaneGenerateVariants> {
	let mut each_attrs = Vec::new();

	let _Some(paren) = paren_option else { return Ok(SaneGenerateVariants::default()) };

	for types_cfg in paren.into_parts().1 {
		match types_cfg {
			GenerateVariantTypesSetting::Attrs(SettingAttrs { attrs, .. }) => {
				each_attrs.extend(attrs.into_inner());
			}
			GenerateVariantTypesSetting::Derive(SettingDerive { kw: _kw, paths, .. }) => {
				let derive_ident = Ident::new("derive", _kw.span);
				let meta_list = (|| Ok(try_parse_quote! { #derive_ident #paths }))().map_err(
					|mut err: Error| {
						let msg = Error::new(
							derive_ident.span(),
							"parse_quote! failed to convert tokens into a MetaList, we tried to merge this ident..",
						);
						err.combine(msg);

						let msg = Error::new(paths.span(), "..with these paths");
						err.combine(msg);

						err
					},
				)?;
				each_attrs.push(Meta::List(meta_list));
			}
		}
	}

	Ok(SaneGenerateVariants {
		kw,
		attrs: each_attrs,
	})
}

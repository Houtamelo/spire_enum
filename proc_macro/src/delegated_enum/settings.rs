use super::*;

mod kw {
	use super::*;
	custom_keyword!(extract_variants);
	custom_keyword!(impl_enum_try_into_variants);
	custom_keyword!(impl_variants_into_enum);
	custom_keyword!(impl_conversions);
}

#[derive(Default)]
pub struct Settings {
	pub extract_variants: Optional<SaneSettingExtractVariants>,
	enum_try_into_variants: Optional<kw::impl_enum_try_into_variants>,
	variants_into_enum: Optional<kw::impl_variants_into_enum>,
	conversions: Optional<kw::impl_conversions>,
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
	let mut sane_settings = Settings::default();

	for setting in setting_list {
		match setting {
			Setting::ExtractVariants(extract_variants) => {
				let sane_extract_variants = sanitize_extract_variants(extract_variants)?;
				assign_unique_or_panic!(sane_settings.extract_variants, sane_extract_variants);
			}
			Setting::ImplVariantsTryFromEnum(kw) => {
				assign_unique_or_panic!(sane_settings.enum_try_into_variants, kw)
			}
			Setting::ImplEnumFromVars(kw) => {
				assign_unique_or_panic!(sane_settings.variants_into_enum, kw)
			}
			Setting::ImplConversions(kw) => {
				assign_unique_or_panic!(sane_settings.conversions, kw)
			}
		}
	}

	match (
		&sane_settings.enum_try_into_variants,
		&sane_settings.variants_into_enum,
		&sane_settings.conversions,
	) {
		| (_, _Some(..), _Some(..)) | (_Some(..), _, _Some(..)) => {
			bail!(sane_settings.conversions => "Setting `conversions` already implies `enum_try_into_variants` and `variants_into_enum`")
		}
		_ => {}
	}

	Ok(sane_settings)
}

#[derive(Parse, ToTokens)]
enum Setting {
	ExtractVariants(SettingExtractVariants),
	ImplVariantsTryFromEnum(kw::impl_enum_try_into_variants),
	ImplEnumFromVars(kw::impl_variants_into_enum),
	ImplConversions(kw::impl_conversions),
}

#[derive(Parse, ToTokens)]
struct SettingExtractVariants {
	kw: kw::extract_variants,
	attrs: Optional<Paren<Punctuated<ExtractVariantsAttrs, Token![,]>>>,
}

#[derive(Parse, ToTokens)]
enum ExtractVariantsAttrs {
	Attrs(SettingAttrs),
	Derive(SettingDerive),
}

#[derive(Default)]
pub struct SaneSettingExtractVariants {
	pub kw: kw::extract_variants,
	pub attrs: Vec<SynMeta>,
}

impl SaneSettingExtractVariants {
	pub fn span(&self) -> Span { self.kw.span() }
}

fn sanitize_extract_variants(input: SettingExtractVariants) -> Result<SaneSettingExtractVariants> {
	let SettingExtractVariants { kw, attrs } = input;
	let mut each_attrs = Vec::new();

	let _Some(paren) = attrs else { return Ok(SaneSettingExtractVariants::default()) };

	for types_cfg in paren.into_parts().1 {
		match types_cfg {
			ExtractVariantsAttrs::Attrs(SettingAttrs { attrs, .. }) => {
				each_attrs.extend(attrs.into_inner());
			}
			ExtractVariantsAttrs::Derive(SettingDerive { kw: _kw, paths, .. }) => {
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
				each_attrs.push(SynMeta::List(meta_list));
			}
		}
	}

	Ok(SaneSettingExtractVariants {
		kw,
		attrs: each_attrs,
	})
}

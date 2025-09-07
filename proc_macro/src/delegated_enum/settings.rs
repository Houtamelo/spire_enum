use syn::parse::discouraged::Speculative;

use super::*;

mod kw {
    use super::*;
    custom_keyword!(inherit_enum_derives);
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

pub fn parse_settings(input_stream: TokenStream) -> Result<Settings> {
    let setting_list = syn::parse2::<InputPunctuated<Setting, Token![,]>>(input_stream)?;
    let mut sane_settings = Settings::default();

    for setting in setting_list.inner {
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

#[derive(ToTokens)]
enum Setting {
    ExtractVariants(SettingExtractVariants),
    ImplVariantsTryFromEnum(kw::impl_enum_try_into_variants),
    ImplEnumFromVars(kw::impl_variants_into_enum),
    ImplConversions(kw::impl_conversions),
}

impl Parse for Setting {
    //noinspection RsUnwrap
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(kw::extract_variants) {
            let kw = input.parse::<kw::extract_variants>()?;

            // Intentional wrap, we should stop the entire parsing if the inner contents are invalid.
            let attrs = input
                .parse::<Optional<Paren<InputPunctuated<ExtractVariantsAttrs, Token![,]>>>>()
                .unwrap();

            Ok(Setting::ExtractVariants(SettingExtractVariants { kw, attrs }))
        } else if let Ok(kw) = input.parse::<kw::impl_enum_try_into_variants>() {
            Ok(Setting::ImplVariantsTryFromEnum(kw))
        } else if let Ok(kw) = input.parse::<kw::impl_variants_into_enum>() {
            Ok(Setting::ImplEnumFromVars(kw))
        } else if let Ok(kw) = input.parse::<kw::impl_conversions>() {
            Ok(Setting::ImplConversions(kw))
        } else {
            Err(input.error(
                "Expected one of `extract_variants`, `impl_enum_try_into_variants`, `impl_variants_into_enum`, or `impl_conversions`."
            ))
        }
    }
}

#[derive(Parse, ToTokens)]
struct SettingExtractVariants {
    kw: kw::extract_variants,
    attrs: Optional<Paren<InputPunctuated<ExtractVariantsAttrs, Token![,]>>>,
}

#[derive(ToTokens)]
enum ExtractVariantsAttrs {
    InheritEnumDerives(kw::inherit_enum_derives),
    Attrs(SettingAttrs),
    Derive(SettingDerive),
}

impl ::parsel::Parse for ExtractVariantsAttrs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut errors = Vec::new();

        {
            let fork = input.fork();
            match fork.parse::<kw::inherit_enum_derives>() {
                Ok(ok) => {
                    input.advance_to(&fork);
                    return Ok(Self::InheritEnumDerives(ok));
                }
                Err(err) => errors.push(format!("Not `inherit_enum_derives`: {err}")),
            }
        }

        {
            let fork = input.fork();
            match fork.parse::<SettingAttrs>() {
                Ok(ok) => {
                    input.advance_to(&fork);
                    return Ok(Self::Attrs(ok));
                }
                Err(err) => errors.push(format!("Not `attrs(list..)`: {err}")),
            }
        }

        {
            let fork = input.fork();
            match fork.parse::<SettingDerive>() {
                Ok(ok) => {
                    input.advance_to(&fork);
                    return Ok(Self::Derive(ok));
                }
                Err(err) => errors.push(format!("Not `derive(list..)`: {err}")),
            }
        }

        Err(Error::new(input.span(), format!("Expected one of:\n\t{}", errors.join("\n\t"))))
    }
}

#[derive(Default)]
pub struct SaneSettingExtractVariants {
    pub kw: kw::extract_variants,
    pub attrs: Vec<SynMeta>,
    pub enum_derives: Optional<kw::inherit_enum_derives>,
}

impl SaneSettingExtractVariants {
    pub fn span(&self) -> Span { self.kw.span() }
}

fn sanitize_extract_variants(input: SettingExtractVariants) -> Result<SaneSettingExtractVariants> {
    let SettingExtractVariants { kw, attrs } = input;

    let _Some(paren) = attrs else { return Ok(SaneSettingExtractVariants::default()) };

    let mut each_attrs = Vec::new();
    let mut enum_derives: Optional<kw::inherit_enum_derives> = _None;

    for types_cfg in paren.into_parts().1.inner {
        match types_cfg {
            ExtractVariantsAttrs::InheritEnumDerives(kw) => {
                assign_unique_or_panic!(enum_derives, kw);
            }
            ExtractVariantsAttrs::Attrs(SettingAttrs { kw: _, attrs }) => {
                each_attrs.extend(attrs.into_inner().inner);
            }
            ExtractVariantsAttrs::Derive(SettingDerive { kw: _kw, paths }) => {
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
        enum_derives,
    })
}

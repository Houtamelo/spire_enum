use kw::{mod_name as kw_mod_name, ty_name as kw_ty_name};
use syn::parse_quote;

mod kw {
    syn::custom_keyword!(ty_name);
    syn::custom_keyword!(mod_name);
}

pub mod discriminant_to_generic;
pub mod variant_type_to_generic;
pub mod variant_type_to_variant_type;

use super::*;

fn var_to_field_ident(ident: &Ident) -> Ident {
    let str = ident.to_string().to_case(Case::Snake);
    Ident::new_raw(&str, Span::call_site())
}

#[derive(Default)]
struct SaneTableMetas {
    syn_metas: Vec<SynMeta>,
    cfg_metas: Vec<CfgMeta>,
    ty_name: Optional<SettingTypeName>,
    mod_name: Optional<SettingModName>,
}

#[derive(Parse, ToTokens)]
enum TableMeta {
    TypeName(SettingTypeName),
    ModuleName(SettingModName),
    Derive(SettingDerive),
    Attrs(SettingAttrs),
}

#[derive(Parse, ToTokens)]
struct SettingTypeName {
    kw: kw_ty_name,
    eq_token: Token![=],
    name: Ident,
}

#[derive(Parse, ToTokens)]
struct SettingModName {
    kw: kw_mod_name,
    eq_token: Token![=],
    name: Ident,
}

fn parse_table_metas(input: TokenStream1) -> Result<SaneTableMetas> {
    let input_attrs = syn::parse::<InputPunctuated<Meta<TableMeta>, Token![,]>>(input)?;

    let mut sane = SaneTableMetas::default();
    let (syn_metas, cfg_metas, custom_attrs) = split_input_metas(input_attrs.inner);
    sane.syn_metas = syn_metas;
    sane.cfg_metas = cfg_metas;

    for meta in custom_attrs {
        match meta {
            TableMeta::TypeName(ty_name) => {
                assign_unique_or_panic!(sane.ty_name, ty_name);
            }
            TableMeta::ModuleName(mod_name) => {
                assign_unique_or_panic!(sane.mod_name, mod_name);
            }
            TableMeta::Derive(SettingDerive { kw, paths }) => {
                let syn_meta: SynMeta = parse_quote! { #kw #paths };
                sane.syn_metas.push(syn_meta);
            }
            TableMeta::Attrs(SettingAttrs { kw: _, attrs }) => {
                sane.syn_metas.extend(attrs.into_inner().inner.into_iter())
            }
        }
    }

    Ok(sane)
}

fn length_definition<'a>(
    ident: &Ident,
    var_cfgs: impl Iterator<Item = &'a Any<Attribute<CfgMeta>>>,
) -> TokenStream {
    quote! {
        const #ident: usize = {
            let mut count = 0;

            #(
                #var_cfgs
                {
                    count += 1;
                }
            )*
            count
        };
    }
}

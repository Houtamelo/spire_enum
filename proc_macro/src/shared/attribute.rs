use syn::MacroDelimiter;

use super::*;

#[derive(Clone, Parse, ToTokens)]
pub struct Attribute<Meta> {
    pub pound_token: token::Pound,
    pub inner: Bracket<Meta>,
}

impl<Meta> Attribute<Meta> {
    #[allow(unused)]
    pub fn map<M>(self, f: impl FnOnce(Meta) -> M) -> Attribute<M> {
        let Self { pound_token, inner } = self;
        let (bracket_token, meta) = inner.into_parts();
        Attribute {
            pound_token,
            inner: Bracket::from((bracket_token, f(meta))),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Parse, ToTokens)]
pub enum Meta<T> {
    Custom(T),
    Cfg(CfgMeta),
    Syn(SynMeta),
}

#[allow(clippy::type_complexity)]
pub fn split_input_attrs<T>(
    metas: impl IntoIterator<Item = Attribute<Meta<T>>>,
) -> (Any<Attribute<SynMeta>>, Any<Attribute<CfgMeta>>, Any<Attribute<T>>) {
    let mut syn_metas = Vec::new();
    let mut cfg_metas = Vec::new();
    let mut custom_metas = Vec::new();

    for Attribute { pound_token, inner } in metas {
        let (bracket_token, meta) = inner.into_parts();

        match meta {
            Meta::Custom(c) => custom_metas.push(Attribute {
                pound_token,
                inner: Bracket::from((bracket_token, c)),
            }),
            Meta::Cfg(cfg) => cfg_metas.push(Attribute {
                pound_token,
                inner: Bracket::from((bracket_token, cfg)),
            }),
            Meta::Syn(s) => syn_metas.push(Attribute {
                pound_token,
                inner: Bracket::from((bracket_token, s)),
            }),
        }
    }

    (Any::from(syn_metas), Any::from(cfg_metas), Any::from(custom_metas))
}

pub fn split_input_metas<T>(
    metas: impl IntoIterator<Item = Meta<T>>,
) -> (Vec<SynMeta>, Vec<CfgMeta>, Vec<T>) {
    let mut syn_metas = Vec::new();
    let mut cfg_metas = Vec::new();
    let mut custom_metas = Vec::new();

    for meta in metas {
        match meta {
            Meta::Custom(c) => custom_metas.push(c),
            Meta::Cfg(cfg) => cfg_metas.push(cfg),
            Meta::Syn(s) => syn_metas.push(s),
        }
    }

    (syn_metas, cfg_metas, custom_metas)
}

pub fn parse_cfg_attrs(syn_attrs: Any<Attribute<SynMeta>>) -> Any<Attribute<CfgMeta>> {
    syn_attrs
        .into_inner()
        .into_iter()
        .filter_map(parse_syn_attr_as_cfg)
        .collect()
}

pub fn parse_syn_attr_as_cfg(
    Attribute { pound_token, inner }: Attribute<SynMeta>,
) -> Option<Attribute<CfgMeta>> {
    let (bracket, meta) = inner.into_parts();
    match meta {
        SynMeta::List(list) => {
            let path = list.path;
            let kw: kw_cfg = parsel::parse2(quote::quote!(#path)).ok()?;

            let paren = match list.delimiter {
                MacroDelimiter::Paren(paren) => paren,
                _ => Default::default(),
            };

            let tokens = Paren::from((paren, list.tokens));

            Some(Attribute {
                pound_token,
                inner: Bracket::from((bracket, CfgMeta { kw, tokens })),
            })
        }
        _ => None,
    }
}

#[derive(Clone, Parse, ToTokens)]
pub struct CfgMeta {
    pub kw: kw_cfg,
    pub tokens: Paren<TokenStream>,
}

pub use kw::cfg as kw_cfg;

mod kw {
    syn::custom_keyword!(cfg);
}

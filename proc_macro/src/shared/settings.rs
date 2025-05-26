use super::*;

/// Shorthand for `attrs = [derive(...)]`
#[derive(Parse, ToTokens)]
pub struct SettingDerive {
    pub kw: kw_derive,
    pub paths: Paren<InputPunctuated<Path, Token![,]>>,
}

#[derive(Parse, ToTokens)]
pub struct SettingAttrs {
    pub kw: kw_attrs,
    pub attrs: Paren<InputPunctuated<SynMeta, Token![,]>>,
}

pub use kw::{attrs as kw_attrs, derive as kw_derive};

mod kw {
    use super::*;
    custom_keyword!(derive);
    custom_keyword!(attrs);
}

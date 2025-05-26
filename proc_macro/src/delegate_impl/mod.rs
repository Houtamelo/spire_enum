mod inherent_impl;
mod shared;
mod trait_impl;

use inherent_impl::InputImplInherent;
use shared::*;
use trait_impl::InputImplTrait;

use super::*;

#[derive(Parse, ToTokens)]
pub enum InputImpl {
    Inherent(Box<InputImplInherent>),
    Trait(Box<InputImplTrait>),
}

pub fn run(input: TokenStream1) -> Result<TokenStream> {
    let input = syn::parse::<InputImpl>(input)?;
    match input {
        InputImpl::Inherent(input) => inherent_impl::run(*input),
        InputImpl::Trait(input) => trait_impl::run(*input),
    }
}

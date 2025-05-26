use std::ops::{Deref, DerefMut};

use super::*;

#[derive(ToTokens, Clone)]
pub struct InputPunctuated<T, P> {
    pub inner: Punctuated<T, P>,
}

impl<T, P> Deref for InputPunctuated<T, P> {
    type Target = Punctuated<T, P>;

    fn deref(&self) -> &Self::Target { &self.inner }
}

impl<T, P> DerefMut for InputPunctuated<T, P> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

impl<T: Parse, P: Parse> Parse for InputPunctuated<T, P> {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut inner = Punctuated::new();

        loop {
            match input.parse::<T>() {
                Ok(ok) => inner.push_value(ok),
                Err(..) => break,
            }

            match input.parse::<P>() {
                Ok(ok) => inner.push_punct(ok),
                Err(..) => break,
            }
        }

        Ok(InputPunctuated { inner })
    }
}

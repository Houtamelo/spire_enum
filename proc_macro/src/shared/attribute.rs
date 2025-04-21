use super::*;

#[derive(Clone, Parse, ToTokens)]
pub struct InputAttribute<T = Meta> {
	pub pound_token: token::Pound,
	pub inner: Bracket<T>,
}

impl IntoSyn for InputAttribute<Meta> {
	type Target = Attribute;

	fn into_syn(self) -> Attribute {
		let (bracket_token, meta) = self.inner.into_parts();

		Attribute {
			pound_token: self.pound_token,
			style: AttrStyle::Outer,
			bracket_token,
			meta,
		}
	}
}

impl<T> InputAttribute<T> {
	#[allow(unused)]
	pub fn map<M>(self, f: impl FnOnce(T) -> M) -> InputAttribute<M> {
		let Self { pound_token, inner } = self;
		let (bracket_token, meta) = inner.into_parts();
		InputAttribute {
			pound_token,
			inner: Bracket::from((bracket_token, f(meta))),
		}
	}
}

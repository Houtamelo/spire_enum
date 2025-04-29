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
	Syn(SynMeta),
}

pub fn split_input_attrs<T>(
	metas: impl IntoIterator<Item = Attribute<Meta<T>>>,
) -> (Any<Attribute<SynMeta>>, Any<Attribute<T>>) {
	let mut syn_metas = Vec::new();
	let mut custom_metas = Vec::new();

	for Attribute { pound_token, inner } in metas {
		let (bracket_token, meta) = inner.into_parts();

		match meta {
			Meta::Custom(c) => {
				custom_metas.push(Attribute {
					pound_token,
					inner: Bracket::from((bracket_token, c)),
				})
			}
			Meta::Syn(s) => {
				syn_metas.push(Attribute {
					pound_token,
					inner: Bracket::from((bracket_token, s)),
				})
			}
		}
	}

	(Any::from(syn_metas), Any::from(custom_metas))
}

pub fn split_input_metas<T>(metas: impl IntoIterator<Item = Meta<T>>) -> (Vec<SynMeta>, Vec<T>) {
	let mut syn_metas = Vec::new();
	let mut custom_metas = Vec::new();

	for meta in metas {
		match meta {
			Meta::Custom(c) => custom_metas.push(c),
			Meta::Syn(s) => syn_metas.push(s),
		}
	}

	(syn_metas, custom_metas)
}

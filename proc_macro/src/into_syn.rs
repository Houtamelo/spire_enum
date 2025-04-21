use super::*;

pub trait IntoSyn {
	type Target;
	fn into_syn(self) -> Self::Target;
}

impl IntoSyn for Any<InputAttribute> {
	type Target = Vec<Attribute>;

	fn into_syn(self) -> Vec<Attribute> {
		self.into_inner()
			.into_iter()
			.map(IntoSyn::into_syn)
			.collect()
	}
}

impl<T: IntoSyn, P: Default> IntoSyn for Punctuated<T, P> {
	type Target = syn::punctuated::Punctuated<<T as IntoSyn>::Target, P>;

	fn into_syn(self) -> Self::Target {
		self.into_inner()
			.into_iter()
			.map(|item| item.into_syn())
			.collect()
	}
}

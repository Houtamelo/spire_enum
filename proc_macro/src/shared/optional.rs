use super::*;

#[derive(Default, Clone, Parse, ToTokens)]
pub enum Optional<T> {
	_Some(T),
	#[default]
	_None,
}

impl<T> IntoSyn for Optional<T> {
	type Target = Option<T>;

	fn into_syn(self) -> Option<T> {
		match self {
			_Some(t) => Some(t),
			_None => None,
		}
	}
}

impl<T> Optional<T> {
	pub fn as_ref(&self) -> Optional<&T> {
		match self {
			_Some(some) => _Some(some),
			_None => _None,
		}
	}
}

use super::*;

#[derive(Clone, Parse, ToTokens)]
pub enum Optional<T> {
    _Some(T),
    _None,
}

#[allow(clippy::derivable_impls)]
impl<T> Default for Optional<T> {
    fn default() -> Self {
        _None
    }
}

impl<T> Optional<T> {
    pub fn is_some(&self) -> bool {
        match self {
            _Some(..) => true,
            _None => false,
        }
    }

    pub fn is_none(&self) -> bool {
        !self.is_some()
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Option<U> {
        match self {
            _Some(t) => Some(f(t)),
            _None => None,
        }
    }

    pub fn as_ref(&self) -> Optional<&T> {
        match self {
            _Some(some) => _Some(some),
            _None => _None,
        }
    }
}

impl<T: CollectIdents> CollectIdents for Optional<T> {
    fn collect_idents(&self, map: &mut IdentMap) {
        if let _Some(t) = self {
            collect!(map, t);
        }
    }
}

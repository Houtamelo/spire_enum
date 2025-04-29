mod expressions;
mod generics;
mod items;
mod paths;
mod patterns;
mod types;

use std::collections::HashSet;

use syn::*;

#[cfg(test)] mod tests;

#[derive(Debug, Clone, Default)]
pub struct IdentMap {
	pub tys: HashSet<String>,
	pub traits: HashSet<String>,
	pub lifetimes: HashSet<String>,
	pub constants: HashSet<String>,
	pub ambiguous_paths: HashSet<String>,
}

impl IdentMap {
	pub fn new(input: &impl CollectIdents) -> Self {
		let mut map = Self::default();
		input.collect_idents(&mut map);
		map.filter_solved_ambiguous();
		map
	}

	pub fn filter_solved_ambiguous(&mut self) {
		self.ambiguous_paths
			.extract_if(|id| {
				// Dont check lifetimes, paths don't contain `'`
				self.tys.contains(id) || self.traits.contains(id) || self.constants.contains(id)
			})
			.count();
	}

	pub fn insert_ty(&mut self, id: &Ident) { self.tys.insert(id.to_string()); }
	pub fn insert_trait(&mut self, id: &Ident) { self.traits.insert(id.to_string()); }
	pub fn insert_lifetime(&mut self, id: &Ident) { self.lifetimes.insert(id.to_string()); }
	pub fn insert_constant(&mut self, id: &Ident) { self.constants.insert(id.to_string()); }
	pub fn insert_ambiguous(&mut self, id: &Ident) { self.ambiguous_paths.insert(id.to_string()); }
}

macro_rules! match_collect {
	($Map:expr, $This: ident => $Enum: ident { $($Var: ident),* $(,)? .. }) => {{
	    match $This {
			$( $Enum::$Var(var) => var.collect_idents($Map), )*
			_ => {},
		}
    }};

	($Map:expr, $This: ident => $Enum: ident { $($Var: ident),* $(,)? ..panic }) => {{
	    match $This {
			$( $Enum::$Var(var) => var.collect_idents($Map), )*
			_ => panic!("Unsupported variant: {:?}", $This),
		}
    }};

	($Map:expr, $This: ident => $Enum: ident { $($Var: ident),* $(,)? }) => {{
	    match $This {
			$( $Enum::$Var(var) => var.collect_idents($Map), )*
		}
    }};
}

macro_rules! collect {
    ($Map:expr, $($Expr: expr),*) => {{
	    $( $Expr.collect_idents($Map); )*
    }};
}

pub(crate) use collect;
pub(crate) use match_collect;

pub trait CollectIdents {
	fn collect_idents(&self, map: &mut IdentMap);
}

impl CollectIdents for Lifetime {
	fn collect_idents(&self, map: &mut IdentMap) {
		let Self {
			apostrophe: _,
			ident,
		} = self;
		map.insert_lifetime(ident);
	}
}

impl<T: CollectIdents> CollectIdents for Option<T> {
	fn collect_idents(&self, map: &mut IdentMap) {
		if let Some(t) = self {
			collect!(map, t);
		}
	}
}

impl<T: CollectIdents> CollectIdents for Box<T> {
	fn collect_idents(&self, map: &mut IdentMap) {
		let t = &**self;
		collect!(map, t);
	}
}

impl<T: CollectIdents, Token> CollectIdents for punctuated::Punctuated<T, Token> {
	fn collect_idents(&self, map: &mut IdentMap) {
		for item in self {
			collect!(map, item);
		}
	}
}

impl<T: CollectIdents> CollectIdents for Vec<T> {
	fn collect_idents(&self, map: &mut IdentMap) {
		for item in self {
			collect!(map, item);
		}
	}
}

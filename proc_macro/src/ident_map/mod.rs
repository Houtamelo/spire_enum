mod expressions;
mod generics;
mod items;
mod paths;
mod patterns;
mod types;

use std::{cell::UnsafeCell, collections::HashSet};

use syn::*;

use super::*;

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
		use std::mem::swap;

		let mut this = Self::default();

		unsafe { rw_cache(Self::clear) }
		input.collect_idents();
		unsafe {
			rw_cache(|map| {
				map.filter_solved_ambiguous();

				swap(&mut this.tys, &mut map.tys);
				swap(&mut this.traits, &mut map.traits);
				swap(&mut this.lifetimes, &mut map.lifetimes);
				swap(&mut this.constants, &mut map.constants);
				swap(&mut this.ambiguous_paths, &mut map.ambiguous_paths);
			});
		}

		this
	}

	#[allow(unused)]
	pub fn add(&mut self, input: &impl CollectIdents) {
		debug_assert_ne!(self as *mut _, LOCAL_IDENTS_MAP.with(|m| m.get()));

		unsafe { rw_cache(Self::clear) }
		input.collect_idents();
		unsafe {
			rw_cache(|map| {
				map.filter_solved_ambiguous();

				self.tys.extend(map.tys.drain());
				self.traits.extend(map.traits.drain());
				self.lifetimes.extend(map.lifetimes.drain());
				self.constants.extend(map.constants.drain());
				self.ambiguous_paths.extend(map.ambiguous_paths.drain());
			});
		}
	}

	pub fn clear(&mut self) {
		self.tys.clear();
		self.traits.clear();
		self.lifetimes.clear();
		self.constants.clear();
		self.ambiguous_paths.clear();
	}

	fn filter_solved_ambiguous(&mut self) {
		self.ambiguous_paths
			.extract_if(|id| {
				// Dont check lifetimes, paths don't contain `'`
				self.tys.contains(id) || self.traits.contains(id) || self.constants.contains(id)
			})
			.count();
	}
}

thread_local! {
	static LOCAL_IDENTS_MAP: UnsafeCell<IdentMap> = UnsafeCell::default();
}

/// # Safety
/// `f` must not call mut_local
/// (which also means it cannot call any implementation of `CollectIdents`, Self::add, Self::new)
unsafe fn rw_cache<Ret>(f: impl FnOnce(&mut IdentMap) -> Ret) -> Ret {
	LOCAL_IDENTS_MAP.with(|map| unsafe { f(&mut *map.get()) })
}

fn cache_ty(id: &Ident) { unsafe { rw_cache(|m| m.tys.insert(id.to_string())) }; }
fn cache_trait(id: &Ident) { unsafe { rw_cache(|m| m.traits.insert(id.to_string())) }; }
fn cache_lifetime(id: &Ident) { unsafe { rw_cache(|m| m.lifetimes.insert(id.to_string())) }; }
fn cache_constant(id: &Ident) { unsafe { rw_cache(|m| m.constants.insert(id.to_string())) }; }
fn cache_ambiguous(id: &Ident) {
	unsafe { rw_cache(|m| m.ambiguous_paths.insert(id.to_string())) };
}

macro_rules! match_collect {
	($This: ident => $Enum: ident { $($Var: ident),* $(,)? .. }) => {{
	    match $This {
			$( $Enum::$Var(var) => var.collect_idents(), )*
			_ => {},
		}
    }};

	($This: ident => $Enum: ident { $($Var: ident),* $(,)? ..panic }) => {{
	    match $This {
			$( $Enum::$Var(var) => var.collect_idents(), )*
			_ => panic!("Unsupported variant: {:?}", $This),
		}
    }};

	($This: ident => $Enum: ident { $($Var: ident),* $(,)? }) => {{
	    match $This {
			$( $Enum::$Var(var) => var.collect_idents(), )*
		}
    }};
}

macro_rules! iter_collect {
	($expr:expr) => {{
		for item in $expr {
			item.collect_idents();
		}
	}};

	($Item:ident. $Field:ident in $expr:expr) => {{
		for $Item in $expr {
			$Item.$Field.collect_idents();
		}
	}};
}

macro_rules! collect {
    ($($Expr: expr),*) => {{
	    $( $Expr.collect_idents(); )*
    }};
}

use collect;
use iter_collect;
use match_collect;

pub trait CollectIdents {
	fn collect_idents(&self);
}

impl CollectIdents for Lifetime {
	fn collect_idents(&self) {
		let Self {
			apostrophe: _,
			ident,
		} = self;
		cache_lifetime(ident);
	}
}

impl<T: CollectIdents> CollectIdents for Option<T> {
	fn collect_idents(&self) {
		if let Some(t) = self {
			collect!(t);
		}
	}
}

impl<T: CollectIdents> CollectIdents for Box<T> {
	fn collect_idents(&self) {
		let t = &**self;
		collect!(t);
	}
}

impl<T: CollectIdents> CollectIdents for Optional<T> {
	fn collect_idents(&self) {
		if let _Some(t) = self {
			collect!(t);
		}
	}
}

impl<T: CollectIdents, Token> CollectIdents for punctuated::Punctuated<T, Token> {
	fn collect_idents(&self) {
		for item in self {
			collect!(item);
		}
	}
}

impl<T: CollectIdents> CollectIdents for Vec<T> {
	fn collect_idents(&self) {
		for item in self {
			collect!(item);
		}
	}
}

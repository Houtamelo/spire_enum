#![doc = include_str!("../../README.md")]

extern crate proc_macro;

mod delegate_impl;
mod delegated_enum;
mod ident_map;
mod into_syn;
mod macros;
mod shared;

use ident_map::*;
use into_syn::*;
use itertools::Itertools;
use macros::*;
use parsel::{Parse, ToTokens, ast::*, try_parse_quote};
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use shared::*;
use syn::{
	AttrStyle,
	Attribute,
	Block,
	ConstParam,
	Error,
	Expr,
	ExprClosure,
	FnArg,
	GenericParam,
	Lifetime,
	LifetimeParam,
	Meta,
	Pat,
	PatIdent,
	PatType,
	Path,
	Receiver,
	Result,
	Type,
	TypeGroup,
	TypeParam,
	TypeParamBound,
	TypeParen,
	TypePath,
	TypePtr,
	TypeReference,
	TypeTuple,
	Visibility,
	WhereClause,
	WherePredicate,
	custom_keyword,
	parse::ParseStream,
	spanned::Spanned,
};

/// See the [crate-level](crate) documentation
#[proc_macro_attribute]
pub fn delegated_enum(settings_stream: TokenStream1, enum_stream: TokenStream1) -> TokenStream1 {
	delegated_enum::run(settings_stream, enum_stream)
		.unwrap_or_else(|err| err.into_compile_error())
		.into()
}

/// See the [crate-level](crate) documentation
#[proc_macro_attribute]
pub fn delegate_impl(_input_stream: TokenStream1, impl_stream: TokenStream1) -> TokenStream1 {
	delegate_impl::run(impl_stream)
		.unwrap_or_else(|err| err.into_compile_error())
		.into()
}

fn delegate_macro_ident(enum_ident: &Ident) -> Ident {
	use convert_case::{Case, Casing};
	let mut ident = enum_ident.to_string();
	ident = ident.to_case(Case::Snake);
	ident.insert_str(0, "delegate_");
	Ident::new(&ident, enum_ident.span())
}

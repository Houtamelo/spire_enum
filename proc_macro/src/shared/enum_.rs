use super::*;

#[derive(Parse, ToTokens)]
pub struct Enum<VarAttr, FieldsAttr> {
	pub attrs: Any<Attribute<SynMeta>>,
	pub vis: Visibility,
	pub enum_token: Token![enum],
	pub ident: Ident,
	pub generics: Optional<InputGenerics>,
	pub where_clause: Optional<WhereClause>,
	pub variants: Brace<Punctuated<Var<VarAttr, FieldsAttr>, Token![,]>>,
}

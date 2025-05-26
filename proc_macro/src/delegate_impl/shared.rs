use super::*;

#[derive(Parse, ToTokens)]
pub enum InputImplItem {
    Const(Box<InputImplItemConst>),
    Fn(Box<InputImplItemFn>),
    Type(Box<InputImplItemAssocType>),
    Macro(Box<syn::ImplItemMacro>),
}

#[derive(Parse, ToTokens)]
pub struct InputImplItemFn {
    pub attrs: Any<Attribute<SynMeta>>,
    pub vis:   Visibility,
    pub sig:   InputFnSignature,
    pub body:  InputImplItemFnBody,
}

#[derive(Parse, ToTokens)]
pub enum InputImplItemFnBody {
    Block(Block),
    SemiColon(Token![;]),
}

#[derive(Parse, ToTokens)]
pub struct InputImplItemConst {
    pub attrs: Any<Attribute<SynMeta>>,
    pub vis: Visibility,
    pub const_token: Token![const],
    pub ident: Ident,
    pub generics: Optional<InputGenerics>,
    pub colon_token: Token![:],
    pub ty: Type,
    pub body: InputImplItemConstBody,
    pub semi_token: Token![;],
}

#[derive(Parse, ToTokens)]
pub enum InputImplItemConstBody {
    Some(Token![=], Box<Expr>),
    None,
}

#[derive(Parse, ToTokens)]
pub struct InputImplItemAssocType {
    pub attrs: Any<Attribute<SynMeta>>,
    pub type_token: Token![type],
    pub ident: Ident,
    pub generics: Optional<InputGenerics>,
    pub body: InputImplItemAssocTypeBody,
    pub semi_token: Token![;],
}

#[derive(Parse, ToTokens)]
pub enum InputImplItemAssocTypeBody {
    Some(Token![=], Box<Type>),
    None,
}

#[derive(Parse, ToTokens)]
pub struct InputFnSignature {
    pub constness: Optional<Token![const]>,
    pub asyncness: Optional<Token![async]>,
    pub unsafety: Optional<Token![unsafe]>,
    pub abi: Optional<syn::Abi>,
    pub fn_token: Token![fn],
    pub ident: Ident,
    pub generics: Optional<InputGenerics>,
    pub inputs: Paren<InputPunctuated<FnArg, Token![,]>>,
    pub output: syn::ReturnType,
    pub where_clause: Optional<WhereClause>,
}

pub fn find_enum_ident(ty: &Type) -> Option<&Ident> {
    match ty {
        | Type::Group(TypeGroup { elem, .. })
        | Type::Paren(TypeParen { elem, .. })
        | Type::Reference(TypeReference { elem, .. })
        | Type::Ptr(TypePtr { elem, .. }) => find_enum_ident(elem),

        Type::Tuple(TypeTuple { elems, .. }) => elems.first().and_then(find_enum_ident),
        Type::Path(TypePath { path, .. }) => path.segments.last().map(|seg| &seg.ident),
        _ => None,
    }
}

use super::*;

#[derive(Parse, ToTokens)]
pub struct InputImplTrait {
    attrs: Any<Attribute<SynMeta>>,
    defaultness: Optional<Token![default]>,
    unsafety: Optional<Token![unsafe]>,
    impl_token: Token![impl],
    generics: Optional<InputGenerics>,
    not_token: Optional<Token![!]>,
    trait_path: Path,
    for_token: Token![for],
    self_ty: Type,
    where_clause: Optional<WhereClause>,
    items: Brace<Any<InputImplItem>>,
}

pub fn run(input: InputImplTrait) -> Result<TokenStream> {
    let sane = sanitize_input(input)?;
    generate_output(sane)
}

struct SaneImplTrait {
    attrs: Any<Attribute<SynMeta>>,
    defaultness: Optional<Token![default]>,
    unsafety: Optional<Token![unsafe]>,
    impl_token: Token![impl],
    generics: Optional<SaneGenerics>,
    not_token: Optional<Token![!]>,
    trait_path: Path,
    for_token: Token![for],
    self_ty: Type,
    items: Brace<Any<SaneItem>>,
}

fn sanitize_input(input: InputImplTrait) -> Result<SaneImplTrait> {
    let InputImplTrait {
        attrs,
        defaultness,
        unsafety,
        impl_token,
        generics,
        not_token,
        trait_path,
        for_token,
        self_ty,
        where_clause,
        items,
    } = input;

    let (brace, item_list) = items.into_parts();

    let generics = sanitize_generics(generics, where_clause)?;

    let sane_items = item_list.into_iter().map(sanitize_item).try_collect()?;

    Ok(SaneImplTrait {
        attrs,
        defaultness,
        unsafety,
        impl_token,
        generics,
        not_token,
        trait_path,
        for_token,
        self_ty,
        items: Brace::from((brace, sane_items)),
    })
}

enum SaneItem {
    AssocType(SaneAssocType),
    AssocConst(SaneAssocConst),
    Method(SaneMethod),
    FnWithExplicitImpl(Box<InputImplItemFn>),
}

fn sanitize_item(item: InputImplItem) -> Result<SaneItem> {
    match item {
        InputImplItem::Fn(func) => sanitize_fn(func),
        InputImplItem::Type(assoc_type) => {
            sanitize_assoc_type(*assoc_type).map(SaneItem::AssocType)
        }
        InputImplItem::Const(constant) => sanitize_assoc_const(*constant).map(SaneItem::AssocConst),
        InputImplItem::Macro(mac) => {
            bail!(mac => "Expected function, associated type, or associated constant.\n\
				Help: Macros aren't supported in impl blocks that have the `delegate_impl` attribute.")
        }
    }
}

#[derive(ToTokens)]
struct SaneAssocConst {
    attrs: Any<Attribute<SynMeta>>,
    vis: Visibility,
    const_token: Token![const],
    ident: Ident,
    generics: Optional<InputGenerics>,
    colon_token: Token![:],
    ty: Type,
    eq_token: Token![=],
    value: Box<Expr>,
    semi_token: Token![;],
}

fn sanitize_assoc_const(input: InputImplItemConst) -> Result<SaneAssocConst> {
    let InputImplItemConst {
        attrs,
        vis,
        const_token,
        ident,
        generics,
        colon_token,
        ty,
        body,
        semi_token,
    } = input;

    match body {
        InputImplItemConstBody::Some(eq_token, value) => Ok(SaneAssocConst {
            attrs,
            vis,
            const_token,
            ident,
            generics,
            colon_token,
            ty,
            eq_token,
            value,
            semi_token,
        }),
        InputImplItemConstBody::None => {
            bail!(ident => "Delegating associated constants is not possible, please provide the value.",
				semi_token => "Help: Expected value before this semi-colon")
        }
    }
}

#[derive(ToTokens)]
struct SaneAssocType {
    attrs: Any<Attribute<SynMeta>>,
    type_token: Token![type],
    ident: Ident,
    generics: Optional<InputGenerics>,
    eq_token: Token![=],
    ty: Box<Type>,
    semi_token: Token![;],
}

fn sanitize_assoc_type(input: InputImplItemAssocType) -> Result<SaneAssocType> {
    let InputImplItemAssocType {
        attrs,
        type_token,
        ident,
        generics,
        body,
        semi_token,
    } = input;

    match body {
        InputImplItemAssocTypeBody::Some(eq_token, ty) => Ok(SaneAssocType {
            attrs,
            type_token,
            ident,
            generics,
            eq_token,
            ty,
            semi_token,
        }),
        InputImplItemAssocTypeBody::None => {
            bail!(ident => "Delegating associated types is not possible, please provide the type.",
				semi_token => "Help: Expected type before this semi-colon")
        }
    }
}

fn sanitize_fn(input: Box<InputImplItemFn>) -> Result<SaneItem> {
    match input.body {
        InputImplItemFnBody::Block(..) => Ok(SaneItem::FnWithExplicitImpl(input)),
        InputImplItemFnBody::SemiColon(_semi_token) => Ok(SaneItem::Method(SaneMethod {
            attrs: input.attrs,
            vis: input.vis,
            sig: sanitize_method_signature(input.sig)?,
            _semi_token,
        })),
    }
}

fn generate_output(sane: SaneImplTrait) -> Result<TokenStream> {
    let SaneImplTrait {
        attrs,
        defaultness,
        unsafety: impl_unsafety,
        impl_token,
        generics: impl_generics,
        not_token,
        trait_path,
        for_token,
        self_ty,
        items,
    } = sane;

    let (impl_generics, impl_where_clause) = impl_generics.into_pair();

    let macro_ident = {
        let enum_ident = find_enum_ident(&self_ty)
            .ok_or_else(|| Error::new(self_ty.span(), "Could not find main ident in this type."))?;

        delegate_macro_ident(enum_ident)
    };

    let items = items
        .into_inner()
        .into_iter()
        .map(|item| match item {
            SaneItem::AssocType(ty) => Ok(ty.to_token_stream()),
            SaneItem::AssocConst(cn) => Ok(cn.to_token_stream()),
            SaneItem::FnWithExplicitImpl(explicit) => Ok(explicit.to_token_stream()),
            SaneItem::Method(method) => sane_method_output(method, &macro_ident),
        })
        .try_collect::<_, Vec<_>, _>()?;

    Ok(quote! {
        #attrs
        #defaultness #impl_unsafety #impl_token #impl_generics
        #not_token #trait_path #for_token #self_ty #impl_where_clause {
            #(#items)*
        }
    })
}

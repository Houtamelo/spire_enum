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
        InputImplItemConstBody::Some(eq_token, value) => {
            Ok(SaneAssocConst {
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
            })
        }
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
        InputImplItemAssocTypeBody::Some(eq_token, ty) => {
            Ok(SaneAssocType {
                attrs,
                type_token,
                ident,
                generics,
                eq_token,
                ty,
                semi_token,
            })
        }
        InputImplItemAssocTypeBody::None => {
            bail!(ident => "Delegating associated types is not possible, please provide the type.",
				semi_token => "Help: Expected type before this semi-colon")
        }
    }
}

struct SaneMethod {
    attrs: Any<Attribute<SynMeta>>,
    vis: Visibility,
    sig: SaneMethodSignature,
    _semi_token: Token![;],
}

fn sanitize_fn(input: Box<InputImplItemFn>) -> Result<SaneItem> {
    match input.body {
        InputImplItemFnBody::Block(..) => Ok(SaneItem::FnWithExplicitImpl(input)),
        InputImplItemFnBody::SemiColon(_semi_token) => {
            Ok(SaneItem::Method(SaneMethod {
                attrs: input.attrs,
                vis: input.vis,
                sig: sanitize_method_signature(input.sig)?,
                _semi_token,
            }))
        }
    }
}

struct SaneMethodSignature {
    constness: Optional<Token![const]>,
    asyncness: Optional<Token![async]>,
    unsafety: Optional<Token![unsafe]>,
    abi: Optional<syn::Abi>,
    fn_token: Token![fn],
    ident: Ident,
    generics: Optional<InputGenerics>,
    paren_token: syn::token::Paren,
    receiver: Receiver,
    other_inputs: Vec<SaneNonReceiverFnArg>,
    output: syn::ReturnType,
    where_clause: Optional<WhereClause>,
}

fn sanitize_method_signature(input: InputFnSignature) -> Result<SaneMethodSignature> {
    let InputFnSignature {
        constness,
        asyncness,
        unsafety,
        abi,
        fn_token,
        ident,
        generics,
        inputs,
        output,
        where_clause,
    } = input;

    let (paren_token, inputs) = inputs.into_parts();
    let mut inputs_iter = inputs.inner.into_iter();

    let Some(FnArg::Receiver(receiver)) = inputs_iter.next() else {
        bail!(ident => "Expected function to have a receiver.\n\
			Help: To delegate the implementation to the variants, we need `Self`(the enum) as an argument.")
    };

    let other_inputs = inputs_iter
        .map(|arg| {
            match arg {
                FnArg::Receiver(other_receiver) => {
                    bail!(other_receiver => "Expected exactly one receiver.",
                        receiver => "First receiver declared here"
                    );
                }
                FnArg::Typed(pat_type) => sanitize_fn_arg(pat_type),
            }
        })
        .try_collect()?;

    Ok(SaneMethodSignature {
        constness,
        asyncness,
        unsafety,
        abi,
        fn_token,
        ident,
        generics,
        paren_token,
        receiver,
        other_inputs,
        output,
        where_clause,
    })
}

struct SaneNonReceiverFnArg {
    attrs: Vec<SynAttribute>,
    pat_ident: PatIdent,
    colon_token: Token![:],
    ty: Box<Type>,
}

fn sanitize_fn_arg(arg: PatType) -> Result<SaneNonReceiverFnArg> {
    let PatType {
        attrs,
        pat,
        colon_token,
        ty,
    } = arg;

    match *pat {
        Pat::Ident(pat_ident) => {
            Ok(SaneNonReceiverFnArg {
                attrs,
                pat_ident,
                colon_token,
                ty,
            })
        }
        other => {
            bail!(other => "Patterns in parameters aren't allowed, \
				please use a plain identifier (e.g: `foo: Ty`).")
        }
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
        .map(|item| {
            match item {
                SaneItem::AssocType(ty) => Ok(ty.to_token_stream()),
                SaneItem::AssocConst(cn) => Ok(cn.to_token_stream()),
                SaneItem::FnWithExplicitImpl(explicit) => Ok(explicit.to_token_stream()),
                SaneItem::Method(method) => sane_method_output(method, &macro_ident),
            }
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

fn sane_method_output(method: SaneMethod, macro_ident: &Ident) -> Result<TokenStream> {
    let SaneMethod {
        attrs,
        vis,
        sig,
        _semi_token: _,
    } = method;

    let SaneMethodSignature {
        constness,
        asyncness,
        unsafety: fn_unsafety,
        abi,
        fn_token,
        ident: fn_ident,
        generics: fn_generics,
        paren_token,
        receiver,
        other_inputs,
        output,
        where_clause: fn_where_clause,
    } = sig;

    let attrs = attrs.iter();

    let other_inputs_tt = other_inputs.iter().map(
        |SaneNonReceiverFnArg {
             attrs,
             pat_ident,
             colon_token,
             ty,
         }| quote! { #(#attrs)* #pat_ident #colon_token #ty },
    );

    let invocation_args = other_inputs.iter().map(
        |SaneNonReceiverFnArg {
             attrs: _,
             pat_ident: PatIdent { ident, .. },
             colon_token: _,
             ty: _,
         }| ident,
    );

    let all_args = quote! { #receiver, #(#other_inputs_tt),* };

    let inputs = Paren::from((paren_token, all_args));

    Ok(quote! {
        #( #attrs )*
        #vis #constness #asyncness #fn_unsafety #abi #fn_token
        #fn_ident #fn_generics #inputs #output #fn_where_clause {
            #macro_ident ! { self.#fn_ident( #(#invocation_args),* ).into() }
        }
    })
}

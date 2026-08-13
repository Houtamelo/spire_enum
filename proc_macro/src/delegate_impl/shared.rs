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
    pub vis: Visibility,
    pub sig: InputFnSignature,
    pub body: InputImplItemFnBody,
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

pub struct SaneMethod {
    pub attrs: Any<Attribute<SynMeta>>,
    pub vis: Visibility,
    pub sig: SaneMethodSignature,
    pub _semi_token: Token![;],
}

pub struct SaneMethodSignature {
    pub constness: Optional<Token![const]>,
    pub asyncness: Optional<Token![async]>,
    pub unsafety: Optional<Token![unsafe]>,
    pub abi: Optional<syn::Abi>,
    pub fn_token: Token![fn],
    pub ident: Ident,
    pub generics: Optional<InputGenerics>,
    pub paren_token: syn::token::Paren,
    pub receiver: ReceiverKind,
    pub other_inputs: Vec<SaneNonReceiverFnArg>,
    pub output: syn::ReturnType,
    pub where_clause: Optional<WhereClause>,
}

pub enum ReceiverKind {
    Std(Receiver),
    NonReceiverWithAttr {
        receiver_attr: Attribute<kw_receiver>,
        arg: SaneNonReceiverFnArg,
        position: usize,
    },
}

pub fn sanitize_method_signature(input: InputFnSignature) -> Result<SaneMethodSignature> {
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

    let mut receiver_opt: Option<ReceiverKind> = None;
    let mut other_inputs = Vec::new();

    for (position, arg) in inputs.inner.into_iter().enumerate() {
        match (arg, &receiver_opt) {
            (FnArg::Receiver(second_receiver), Some(first_receiver)) => {
                bail!(second_receiver => "Expected exactly one receiver.",
                    first_receiver => "First receiver declared here"
                );
            }
            (FnArg::Receiver(first_receiver), None) => {
                receiver_opt = Some(ReceiverKind::Std(first_receiver));
            }
            (FnArg::Typed(mut typed_arg), Some(first_receiver)) => {
                if extract_attr::<kw_receiver>(&mut typed_arg.attrs).is_some() {
                    bail!(typed_arg => "Expected exactly one receiver.",
                        first_receiver => "First receiver declared here"
                    );
                } else {
                    let sanitized = sanitize_fn_arg(typed_arg)?;
                    other_inputs.push(sanitized);
                }
            }
            (FnArg::Typed(mut typed_arg), None) => {
                if let Some(receiver_attr) = extract_attr::<kw_receiver>(&mut typed_arg.attrs) {
                    let sanitized = sanitize_fn_arg(typed_arg)?;
                    receiver_opt = Some(ReceiverKind::NonReceiverWithAttr {
                        receiver_attr,
                        arg: sanitized,
                        position,
                    });
                } else {
                    let sanitized = sanitize_fn_arg(typed_arg)?;
                    other_inputs.push(sanitized);
                }
            }
        }
    }

    let Some(receiver) = receiver_opt else {
        bail!(ident => "Expected function to have a receiver.\n\
			Help: To delegate the implementation to the variants, we need `Self`(the enum) as an argument.")
    };

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

impl ReceiverKind {
    pub fn span(&self) -> Span {
        match self {
            ReceiverKind::Std(std) => std.span(),
            ReceiverKind::NonReceiverWithAttr {
                receiver_attr,
                arg: _,
                position: _,
            } => receiver_attr.span(),
        }
    }
}

pub struct SaneNonReceiverFnArg {
    pub attrs: Vec<SynAttribute>,
    pub pat_ident: PatIdent,
    pub colon_token: Token![:],
    pub ty: Box<Type>,
}

impl ToTokens for SaneNonReceiverFnArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let SaneNonReceiverFnArg {
            attrs,
            pat_ident,
            colon_token,
            ty,
        } = self;
        tokens.extend(quote! { #(#attrs)* #pat_ident #colon_token #ty });
    }
}

pub fn sanitize_fn_arg(arg: PatType) -> Result<SaneNonReceiverFnArg> {
    let PatType {
        attrs,
        pat,
        colon_token,
        ty,
    } = arg;

    match *pat {
        Pat::Ident(pat_ident) => Ok(SaneNonReceiverFnArg {
            attrs,
            pat_ident,
            colon_token,
            ty,
        }),
        other => {
            bail!(other => "Patterns in parameters aren't allowed, \
				please use a plain identifier (e.g: `foo: Ty`).")
        }
    }
}

pub fn sane_method_output(method: SaneMethod, macro_ident: &Ident) -> Result<TokenStream> {
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
    let maybe_await = asyncness.as_ref().map(|_| quote! { . await });

    match receiver {
        ReceiverKind::Std(std) => {
            let other_params_tt = other_inputs
                .iter()
                .map(|other_param| quote! { #other_param });

            let invocation_args = other_inputs
                .iter()
                .map(|other_arg| &other_arg.pat_ident.ident);

            let all_args = quote! { #std, #(#other_params_tt),* };
            let inputs = Paren::from((paren_token, all_args));

            Ok(quote! {
                #( #attrs )*
                #vis #constness #asyncness #fn_unsafety #abi #fn_token
                #fn_ident #fn_generics #inputs #output #fn_where_clause {
                    #macro_ident ! { self => |__this| __this.#fn_ident( #(#invocation_args),* ) #maybe_await .into() }
                }
            })
        }
        ReceiverKind::NonReceiverWithAttr {
            receiver_attr: _,
            arg: receiver_arg,
            position,
        } => {
            let mut params_tt = other_inputs
                .iter()
                .map(|other_param| quote! { #other_param })
                .collect::<Vec<_>>();
            params_tt.insert(position, quote! { #receiver_arg });
            let params_punctuated = quote! { #(#params_tt),* };
            let inputs = Paren::from((paren_token, params_punctuated));
            let receiver_ident = &receiver_arg.pat_ident.ident;

            let mut invocation_args = other_inputs
                .iter()
                .map(|arg| &arg.pat_ident.ident)
                .collect::<Vec<_>>();
            let invocation_receiver_ident: Ident = parse_quote! { _this };
            invocation_args.insert(position, &invocation_receiver_ident);

            Ok(quote! {
                #( #attrs )*
                #vis #constness #asyncness #fn_unsafety #abi #fn_token
                #fn_ident #fn_generics #inputs #output #fn_where_clause {
                    #macro_ident ! {
                        @NON_RECEIVER
                        { #fn_ident } { #receiver_ident => |#invocation_receiver_ident| } { #(#invocation_args),* }
                        #maybe_await .into()
                    }
                }
            })
        }
    }
}

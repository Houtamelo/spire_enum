use syn::parse_quote;

use super::*;

#[derive(ToTokens, Clone)]
pub struct InputGenerics {
    pub lb_token: Token![<],
    pub params:   InputPunctuated<GenericParam, Token![,]>,
    pub rb_token: Token![>],
}

#[derive(Parse, ToTokens)]
pub enum InputGenericParam {
    Lifetime(Box<InputGenericParamLifetime>),
    Type(Box<InputGenericParamType>),
    Const(Box<InputGenericParamConst>),
}

#[allow(unused)]
pub fn new_ty_generic(ident: &Ident, generics: &SaneGenerics) -> Type {
    let gen_args = generics.stream_args();
    parse_quote! { #ident #gen_args }
}

pub fn new_ty_maybe_generic(ident: &Ident, generics: &Optional<SaneGenerics>) -> Type {
    let gen_args = generics.stream_args();
    parse_quote! { #ident #gen_args }
}

#[derive(Clone, Parse, ToTokens)]
pub struct InputGenericParamLifetime {
    pub attrs: Any<Attribute<SynMeta>>,
    pub lifetime: Lifetime,
    pub bounds: InputGenericParamLifetimeBounds,
}

#[derive(Clone, Parse, ToTokens)]
pub enum InputGenericParamLifetimeBounds {
    Some(Token![:], Separated<Lifetime, Token![+]>),
    None,
}

#[derive(Clone, Parse, ToTokens)]
pub struct InputGenericParamType {
    pub attrs: Any<Attribute<SynMeta>>,
    pub ident: Ident,
    pub colon_token: Option<Token![:]>,
    pub bounds: InputPunctuated<TypeParamBound, Token![+]>,
    pub eq_token: Option<Token![=]>,
    pub default: Optional<Type>,
}

#[derive(Clone, Parse, ToTokens)]
pub struct InputGenericParamConst {
    pub attrs: Any<Attribute<SynMeta>>,
    pub const_token: Token![const],
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: Type,
    pub eq_token: Option<Token![=]>,
    pub default: Optional<Expr>,
}

impl Parse for InputGenerics {
    fn parse(input: ParseStream) -> Result<Self> {
        #[allow(unused_qualifications)]
        let syn::Generics {
            lt_token: Some(_left_angle_bracket),
            params,
            gt_token: Some(_right_angle_bracket),
            where_clause: Option::None,
        } = input.parse()?
        else {
            return Err(Error::new(input.span(), "expected generics (no where clause)"));
        };

        Ok(InputGenerics {
            lb_token: _left_angle_bracket,
            params:   InputPunctuated {
                inner: params.into_iter().collect(),
            },
            rb_token: _right_angle_bracket,
        })
    }
}

#[derive(Clone)]
pub struct SaneGenerics {
    pub input: InputGenerics,
    pub where_clause: Optional<WhereClause>,
}

pub fn sanitize_generics(
    generics: Optional<InputGenerics>,
    where_clause: Optional<WhereClause>,
) -> Result<Optional<SaneGenerics>> {
    match (generics, where_clause) {
        (_Some(mut input), where_clause) => {
            input.params.pop_punct();

            Ok(_Some(SaneGenerics {
                input,
                where_clause,
            }))
        }
        (_None, _Some(where_clause)) => {
            bail!(where_clause => "where-clause without generics");
        }
        (_None, _None) => Ok(_None),
    }
}

impl SaneGenerics {
    #[allow(unused)]
    pub fn stream_params(&self) -> TokenStream { self.input.to_token_stream() }

    pub fn stream_params_list(&self) -> TokenStream { self.input.params.to_token_stream() }

    pub fn stream_args(&self) -> TokenStream {
        let Self {
            input: InputGenerics {
                lb_token, rb_token, ..
            },
            ..
        } = self;
        let args_list = self.stream_args_list();
        parse_quote!(#lb_token #args_list #rb_token)
    }

    pub fn stream_args_list(&self) -> TokenStream {
        let args = self.input.params.inner.iter().map(|p| {
            match p {
                GenericParam::Lifetime(lf) => lf.lifetime.to_token_stream(),
                GenericParam::Type(ty) => ty.ident.to_token_stream(),
                GenericParam::Const(cn) => cn.ident.to_token_stream(),
            }
        });

        quote! { #(#args),* }
    }
}

impl Optional<SaneGenerics> {
    #[allow(unused)]
    pub fn stream_params(&self) -> TokenStream {
        self.as_ref()
            .map(SaneGenerics::stream_params)
            .unwrap_or_default()
    }

    pub fn stream_params_list(&self) -> TokenStream {
        self.as_ref()
            .map(SaneGenerics::stream_params_list)
            .unwrap_or_default()
    }

    pub fn stream_args(&self) -> TokenStream {
        self.as_ref()
            .map(SaneGenerics::stream_args)
            .unwrap_or_default()
    }

    pub fn stream_args_list(&self) -> TokenStream {
        self.as_ref()
            .map(SaneGenerics::stream_args_list)
            .unwrap_or_default()
    }

    pub fn into_pair(self) -> (Optional<InputGenerics>, Optional<WhereClause>) {
        match self {
            _Some(generics) => (_Some(generics.input), generics.where_clause),
            _None => Default::default(),
        }
    }

    pub fn as_pair(&self) -> (Optional<&InputGenerics>, Optional<&WhereClause>) {
        match self {
            _Some(generics) => (_Some(&generics.input), generics.where_clause.as_ref()),
            _None => Default::default(),
        }
    }
}

impl CollectIdents for SaneGenerics {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            input: stage0,
            where_clause,
        } = self;
        collect!(map, stage0, where_clause);
    }
}

impl CollectIdents for InputGenerics {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            lb_token: _left_angle_bracket,
            params,
            rb_token: _right_angle_bracket,
        } = self;
        collect!(map, params.inner);
    }
}

impl CollectIdents for InputGenericParamType {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            attrs: _,
            ident,
            colon_token: _,
            bounds,
            eq_token: _,
            default,
        } = self;
        map.insert_ty(ident);
        collect!(map, bounds.inner, default);
    }
}

impl CollectIdents for InputGenericParam {
    fn collect_idents(&self, map: &mut IdentMap) {
        use crate::InputGenericParam;
        match_collect!(map, self => InputGenericParam{Lifetime, Type, Const});
    }
}

impl CollectIdents for InputGenericParamConst {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            attrs: _,
            const_token: _,
            ident,
            colon_token: _,
            ty,
            eq_token: _,
            default: _,
        } = self;
        map.insert_constant(ident);
        collect!(map, ty);
    }
}

impl CollectIdents for InputGenericParamLifetime {
    fn collect_idents(&self, map: &mut IdentMap) {
        let Self {
            attrs: _,
            lifetime,
            bounds,
        } = self;

        collect!(map, lifetime);
        match bounds {
            InputGenericParamLifetimeBounds::Some(_, list) => collect!(map, list),
            InputGenericParamLifetimeBounds::None => {}
        }
    }
}

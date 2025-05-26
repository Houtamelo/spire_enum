mod var_kw {
    use super::*;
    custom_keyword!(dont_extract);
    custom_keyword!(dont_impl_conversions);
    custom_keyword!(delegate_via);
}

use var_kw::{
    delegate_via as kw_delegate_via,
    dont_extract as kw_dont_extract,
    dont_impl_conversions as kw_dont_impl_conversions,
};

use super::*;

#[derive(Parse, ToTokens)]
pub enum VarMeta {
    NoVarType(kw_dont_extract),
    NoConversions(kw_dont_impl_conversions),
    DelegateVia(kw_delegate_via, Box<Paren<ExprClosure>>),
}

pub(super) fn generate_variant_type_definition(
    variant: &SaneVar,
    enum_def: &SaneEnum,
    extra_attrs: &[SynMeta],
    enum_derives: &[&Attribute<SynMeta>],
) -> TokenStream {
    let SaneVar {
        attrs: SaneVariantAttributes { syn_attrs, .. },
        ident: var_ident,
        fields,
        discriminant: _,
        generics,
        explicit_delegator: _,
    } = variant;

    let vis = &enum_def.vis;

    let (generics, where_clause) = generics.as_pair();

    let fields_and_where_clause = match fields {
        SaneVarFields::Named(named) => {
            let fields = named.fields.iter().map(
                |VarFieldNamed {
                     attrs,
                     ident,
                     colon_token,
                     ty,
                 }| {
                    quote! { #attrs pub #ident #colon_token #ty }
                },
            );

            quote! { #where_clause { #(#fields),* } }
        }
        SaneVarFields::Unnamed(unnamed) => {
            let fields = unnamed.fields.iter().map(|VarFieldUnnamed { attrs, ty }| {
                quote! { #attrs pub #ty }
            });

            quote! { ( #(#fields),* ) #where_clause ; }
        }
        SaneVarFields::Unit => quote! { #where_clause ; },
    };

    quote! {
        #(#enum_derives)*
        #syn_attrs
        #( #[#extra_attrs] )*
        #vis struct #var_ident #generics #fields_and_where_clause
    }
}

pub(super) fn generate_enum_variant_definition(
    variant: &SaneVar,
    settings: &Settings,
) -> Result<TokenStream> {
    let SaneVar {
        attrs: SaneVariantAttributes { syn_attrs, .. },
        ident: var_ident,
        fields,
        discriminant,
        generics,
        explicit_delegator: _,
    } = variant;

    let will_type_be_generated = settings.extract_variants.is_some() && variant.allow_extract();

    if will_type_be_generated {
        let args = generics.stream_args();

        let var_ty: Type = {
            (|| Ok(try_parse_quote!(#var_ident #args)))().map_err(|mut err: Error| {
                let msg = Error::new(
                    var_ident.span(),
                    "parse_quote! failed to create a syn::Type, we tried to merge this ident..",
                );
                err.combine(msg);

                let msg = Error::new(args.span(), "..with these generics");
                err.combine(msg);

                err
            })?
        };

        Ok(quote! {
            #syn_attrs
            #var_ident ( #var_ty )
        })
    } else {
        Ok(quote! {
            #syn_attrs
            #var_ident #fields #discriminant
        })
    }
}

pub struct SaneVar {
    pub attrs: SaneVariantAttributes,
    pub ident: Ident,
    pub fields: SaneVarFields,
    pub discriminant: Optional<InputDiscriminant>,
    pub generics: Optional<SaneGenerics>,
    pub explicit_delegator: Optional<ExplicitDelegator>,
}

#[derive(ToTokens)]
pub enum ExplicitDelegator {
    Expr(#[allow(unused)] kw_delegate_via, Box<Paren<ExprClosure>>),
}

impl SaneVar {
    pub fn allow_extract(&self) -> bool { self.attrs.no_var_type.is_none() }
    pub fn allow_generate_conversions(&self) -> bool { self.attrs.no_convert.is_none() }
}

pub(super) fn sanitize_variant(
    variant: Var<Meta<VarMeta>, Meta<kw_delegator>>,
    settings: &Settings,
    enum_generics: &Optional<SaneGenerics>,
) -> Result<SaneVar> {
    let attrs = sanitize_attributes(variant.attrs)?;
    let explicit_delegator = find_delegator(&attrs)?;

    if let _Some(ExplicitDelegator::Expr(_, expr)) = &explicit_delegator {
        if settings.extract_variants.is_some()
            && attrs.no_var_type.is_none()
            && expr.inputs.len() > 1
        {
            bail!(expr => "a type will be generated for this variant, the delegator closure should either \
			    take no parameters or take the variant's type as the single parameter");
        }

        if expr.inputs.len() > variant.fields.len() {
            bail!(expr => "delegator closure should not take more parameters than variant's field count");
        }
    }

    let generics = generics_needed_by_variant(&variant.fields, enum_generics);

    let fields = sanitize_variant_fields(variant.fields)?;
    if let (_Some(a), Some(b)) = (&explicit_delegator, fields.delegator_field_kw()) {
        err_expected_only_one!(a, b)
    }

    Ok(SaneVar {
        attrs,
        ident: variant.ident,
        fields,
        discriminant: variant.discriminant,
        generics,
        explicit_delegator,
    })
}

#[derive(Default)]
pub struct SaneVariantAttributes {
    pub syn_attrs: Any<Attribute<SynMeta>>,
    pub no_var_type: Optional<kw_dont_extract>,
    pub no_convert: Optional<kw_dont_impl_conversions>,
    pub delegate_via: Optional<(kw_delegate_via, Box<Paren<ExprClosure>>)>,
}

fn sanitize_attributes(attrs: Any<Attribute<Meta<VarMeta>>>) -> Result<SaneVariantAttributes> {
    let (syn_attrs, custom_attrs) = split_input_attrs(attrs);

    let mut sane = SaneVariantAttributes {
        syn_attrs,
        ..SaneVariantAttributes::default()
    };

    for attr in custom_attrs {
        match attr.inner.into_inner() {
            VarMeta::NoVarType(kw) => assign_unique_or_panic!(sane.no_var_type, kw),
            VarMeta::NoConversions(kw) => assign_unique_or_panic!(sane.no_convert, kw),
            VarMeta::DelegateVia(kw, expr) => {
                if let _Some((first_kw, _)) = sane.delegate_via {
                    err_expected_only_one!(first_kw, kw);
                } else {
                    sane.delegate_via = _Some((kw, expr));
                }
            }
        }
    }

    Ok(sane)
}

fn find_delegator(attrs: &SaneVariantAttributes) -> Result<Optional<ExplicitDelegator>> {
    match attrs.delegate_via.clone() {
        _Some((kw, expr)) => Ok(_Some(ExplicitDelegator::Expr(kw, expr))),
        _None => Ok(_None),
    }
}

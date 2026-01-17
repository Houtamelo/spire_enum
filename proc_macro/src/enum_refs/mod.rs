#![allow(clippy::too_many_arguments)]
use super::*;

pub fn tokenize_enum_ref<'a>(
    vis: &Visibility,
    enum_ident: &Ident,
    gen_lf_params: &TokenStream,
    gen_lf_args: &TokenStream,
    where_clause: Optional<&WhereClause>,
    var_cfgs: &[&Any<Attribute<CfgMeta>>],
    var_tys: &[&'a Type],
    var_idents: &[&'a Ident],
) -> (Ident, Type, TokenStream) {
    let lf = Lifetime::new("'_r", Span::call_site());
    let ident = format_ident!("{enum_ident}Ref");
    let ty: Type = parse_quote! { #ident::#gen_lf_args };

    let mut def = {
        let docs = docs_tokens(format!(
            "An enum that mirrors the variants of [`{enum_ident}`], except this \
			 holds a reference to the variant's type instead of owning it."
        ));

        quote! {
            #docs
            #vis enum #ident #gen_lf_params #where_clause {
                #(
                    #var_cfgs
                    #var_idents (&#lf #var_tys)
                ),*
            }
        }
    };

    for ((var_cfg, var_ty), var_ident) in var_cfgs.iter().zip(var_tys.iter()).zip(var_idents.iter())
    {
        def.extend(quote! {
            #var_cfg
            impl #gen_lf_params ::core::convert::From<&#lf #var_ty> for #ident #gen_lf_params
            #where_clause
            {
                fn from(var: &#lf #var_ty) -> Self {
                    Self::#var_ident(var)
                }
            }
        });
    }

    (ident, ty, def)
}

pub fn tokenize_enum_mut<'a>(
    vis: &Visibility,
    enum_ident: &Ident,
    gen_lf_params: &TokenStream,
    gen_lf_args: &TokenStream,
    where_clause: Optional<&WhereClause>,
    var_cfgs: &[&Any<Attribute<CfgMeta>>],
    var_tys: &[&'a Type],
    var_idents: &[&'a Ident],
) -> (Ident, Type, TokenStream) {
    let lf = Lifetime::new("'_r", Span::call_site());
    let ident = format_ident!("{enum_ident}Mut");
    let ty: Type = parse_quote! { #ident::#gen_lf_args };

    let mut def = {
        let docs = docs_tokens(format!(
            "An enum that mirrors the variants of [`{enum_ident}`], except this \
            holds mutable references to the variants instead of owning their values."
        ));

        quote! {
            #docs
            #vis enum #ident #gen_lf_params #where_clause {
                #(
                    #var_cfgs
                    #var_idents (&#lf mut #var_tys)
                ),*
            }
        }
    };

    for ((var_cfg, var_ty), var_ident) in var_cfgs.iter().zip(var_tys.iter()).zip(var_idents.iter())
    {
        def.extend(quote! {
            #var_cfg
            impl #gen_lf_params ::core::convert::From<&#lf mut #var_ty> for #ident #gen_lf_params
            #where_clause
            {
                fn from(var: &#lf mut #var_ty) -> Self {
                    Self::#var_ident(var)
                }
            }
        });
    }

    (ident, ty, def)
}

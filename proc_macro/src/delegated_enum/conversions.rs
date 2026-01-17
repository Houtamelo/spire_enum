use quote::TokenStreamExt;

use super::*;

const HELP_CONVERT_1: &str = "Expected variant to have exactly one field. \n\
		Help: For conversions to be implemented, the variant must satisfy one of:\n\
		- Have exactly one field (and that field's type will be the one implementing conversions).\n\
		- Have the setting \"generate type\" turned on (and the generated variant's type \
		will be the one implementing conversions).";

const HELP_CONVERT_2: &str = "Help: If you do not want to generate conversions, \
		add the attribute #[no_convert] to this variant";

pub fn generate_variant_try_from_enum(
    variant: &SaneVar,
    enum_def: &SaneEnum,
    settings: &Settings,
) -> Result<TokenStream> {
    let SaneEnum {
        attrs: _,
        vis: _,
        enum_token: _,
        ident: enum_ident,
        generics: enum_generics,
        _brace,
        variants: _,
        ty: enum_ty,
    } = enum_def;

    let SaneVar {
        attrs:
            SaneVariantAttributes {
                syn_attrs: _,
                cfg_attrs: var_cfg_attrs,
                no_var_type: _,
                no_convert: _,
                delegate_via: _,
            },
        ident: var_ident,
        fields: var_fields,
        discriminant: _,
        generics: var_generics,
        explicit_delegator: _,
    } = variant;

    let lf = Lifetime::new("'_r1", Span::call_site());
    let where_clause = enum_generics.as_pair().1;

    let (gen_params, gen_lf_params) = {
        let gen_params_list = {
            let mut tokens = enum_generics.stream_params_list();
            if !tokens.is_empty() {
                tokens.append_terminated(std::iter::empty::<TokenStream>(), <Token![,]>::default());
            }

            tokens
        };

        let (lb, rb) = match &enum_generics {
            _Some(SaneGenerics {
                input:
                    InputGenerics {
                        lb_token,
                        params: _,
                        rb_token,
                    },
                where_clause: _,
            }) => (lb_token, rb_token),
            _None => (&Default::default(), &Default::default()),
        };

        let (gen_params, gen_lf_params) = if !gen_params_list.is_empty() {
            (quote! { #lb #gen_params_list #rb }, quote! { #lb #lf, #gen_params_list #rb })
        } else {
            (Default::default(), quote! { #lb #lf #rb })
        };

        (gen_params, gen_lf_params)
    };

    let fn_input = Ident::new("_value", Span::call_site());

    let will_type_be_generated = settings.extract_variants.is_some() && variant.allow_extract();

    let (var_ty, if_var_from_enum) = if will_type_be_generated {
        let var_ty = {
            let args = var_generics.stream_args();
            quote! { #var_ident #args }
        };

        let if_var_from_enum = quote! {
            if let #enum_ident::#var_ident(__var) = #fn_input {
                ::core::result::Result::Ok(__var)
            }
        };

        (var_ty, if_var_from_enum)
    } else {
        match var_fields {
            SaneVarFields::Named(named) => {
                if named.fields.len() != 1 {
                    bail!(named => HELP_CONVERT_1, var_ident => HELP_CONVERT_2);
                }

                let VarFieldNamed {
                    attrs: _,
                    ident: field_ident,
                    colon_token: _,
                    ty: var_ty,
                } = &named.fields[0];

                let if_var_from_enum = quote! {
                    if let #enum_ident::#var_ident { #field_ident, .. } = #fn_input {
                        Ok(#field_ident)
                    }
                };

                (quote! { #var_ty }, if_var_from_enum)
            }
            SaneVarFields::Unnamed(unnamed) => {
                if unnamed.fields.len() != 1 {
                    bail!(unnamed => HELP_CONVERT_1, var_ident => HELP_CONVERT_2);
                }

                let VarFieldUnnamed {
                    attrs: _,
                    ty: field_ty,
                } = &unnamed.fields[0];

                let if_var_from_enum = quote! {
                    if let #enum_ident::#var_ident(__var) = #fn_input {
                        ::core::result::Result::Ok(__var)
                    }
                };

                (quote! { #field_ty }, if_var_from_enum)
            }
            SaneVarFields::Unit => {
                bail!(var_fields => HELP_CONVERT_1, var_ident => HELP_CONVERT_2)
            }
        }
    };

    Ok(quote! {
        #var_cfg_attrs
        impl #gen_params ::core::convert::TryFrom<#enum_ty> for #var_ty #where_clause {
            type Error = #enum_ty;

            fn try_from(#fn_input: #enum_ty) -> ::core::result::Result<Self, Self::Error> {
                #if_var_from_enum
                else { ::core::result::Result::Err(#fn_input) }
            }
        }

        #var_cfg_attrs
        impl #gen_lf_params ::core::convert::TryFrom<&#lf #enum_ty> for &#lf #var_ty #where_clause {
            type Error = ();

            fn try_from(#fn_input: &#lf #enum_ty) -> ::core::result::Result<Self, Self::Error> {
                #if_var_from_enum
                else { ::core::result::Result::Err(()) }
            }
        }

        #var_cfg_attrs
        impl #gen_lf_params ::core::convert::TryFrom<&#lf mut #enum_ty> for &#lf mut #var_ty #where_clause {
            type Error = ();

            fn try_from(#fn_input: &#lf mut #enum_ty) -> ::core::result::Result<Self, Self::Error> {
                #if_var_from_enum
                else { ::core::result::Result::Err(()) }
            }
        }

        #var_cfg_attrs
        impl #gen_params ::spire_enum::prelude::FromEnum<#enum_ty> for #var_ty #where_clause {
            fn from_enum(#fn_input: #enum_ty) -> ::core::result::Result<Self, #enum_ty> {
                #if_var_from_enum
                else { ::core::result::Result::Err(#fn_input) }
            }
        }

        #var_cfg_attrs
        impl #gen_lf_params ::spire_enum::prelude::FromEnumRef<#enum_ty> for #var_ty #where_clause {
            fn from_enum_ref<'__ref>(#fn_input: &'__ref #enum_ty) -> ::core::option::Option<&'__ref Self> {
                if let #enum_ident::#var_ident(__var) = #fn_input {
                    ::core::option::Option::Some(__var)
                } else {
                    ::core::option::Option::None
                }
            }
        }

        #var_cfg_attrs
        impl #gen_lf_params ::spire_enum::prelude::FromEnumMut<#enum_ty> for #var_ty #where_clause {
            fn from_enum_mut<'__ref>(#fn_input: &'__ref mut #enum_ty) -> ::core::option::Option<&'__ref mut Self> {
                if let #enum_ident::#var_ident(__var) = #fn_input {
                    ::core::option::Option::Some(__var)
                } else {
                    ::core::option::Option::None
                }
            }
        }
    })
}

pub fn generate_enum_from_variant(
    variant: &SaneVar,
    enum_def: &SaneEnum,
    settings: &Settings,
) -> Result<TokenStream> {
    let SaneEnum {
        attrs: _,
        vis: _,
        enum_token: _,
        ident: enum_ident,
        generics: enum_generics,
        _brace,
        variants: _,
        ty: enum_ty,
    } = enum_def;
    let SaneVar {
        attrs:
            SaneVariantAttributes {
                syn_attrs: _,
                cfg_attrs: var_cfg_attrs,
                no_var_type: _,
                no_convert: _,
                delegate_via: _,
            },
        ident: var_ident,
        fields: var_fields,
        discriminant: _,
        generics: var_generics,
        explicit_delegator: _,
    } = variant;

    let (enum_generics, where_clause) = enum_generics.as_pair();

    let fn_input = Ident::new("_value", Span::call_site());

    let will_type_be_generated = settings.extract_variants.is_some() && variant.allow_extract();

    let (var_ty, ret_enum_from_var) = if will_type_be_generated {
        let var_ty = {
            let args = var_generics.stream_args();
            quote! { #var_ident #args }
        };

        let ret_enum_from_var = quote! {
            #enum_ident::#var_ident(#fn_input)
        };

        (var_ty, ret_enum_from_var)
    } else {
        match var_fields {
            SaneVarFields::Named(named) => {
                if named.fields.len() != 1 {
                    bail!(named => HELP_CONVERT_1, var_ident => HELP_CONVERT_2);
                }

                let VarFieldNamed {
                    attrs: _,
                    ident: field_ident,
                    colon_token: _,
                    ty: var_ty,
                } = &named.fields[0];

                let ret_enum_from_var = quote! {
                    #enum_ident::#var_ident { #field_ident: #fn_input }
                };

                (quote! { #var_ty }, ret_enum_from_var)
            }
            SaneVarFields::Unnamed(unnamed) => {
                if unnamed.fields.len() != 1 {
                    bail!(unnamed => HELP_CONVERT_1, var_ident => HELP_CONVERT_2);
                }

                let VarFieldUnnamed {
                    attrs: _,
                    ty: var_ty,
                } = &unnamed.fields[0];

                let ret_enum_from_var = quote! {
                    #enum_ident::#var_ident(#fn_input)
                };

                (quote! { #var_ty }, ret_enum_from_var)
            }
            SaneVarFields::Unit => {
                bail!(var_fields => HELP_CONVERT_1, var_ident => HELP_CONVERT_2)
            }
        }
    };

    Ok(quote! {
        #var_cfg_attrs
        impl #enum_generics ::core::convert::From::<#var_ty> for #enum_ty #where_clause {
            fn from(#fn_input: #var_ty) -> Self {
                #ret_enum_from_var
            }
        }
    })
}

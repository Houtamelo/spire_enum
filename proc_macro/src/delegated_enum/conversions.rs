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
        ident: enum_ident,
        generics: enum_generics,
        ty: enum_ty,
        ..
    } = enum_def;

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
                input: InputGenerics {
                    lb_token, rb_token, ..
                },
                ..
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

    let var_ident = &variant.ident;
    let fn_input = Ident::new("_value", Span::call_site());

    let will_type_be_generated = settings.extract_variants.is_some() && variant.allow_extract();

    let (var_ty, if_var_from_enum) = if will_type_be_generated {
        let var_ty = {
            let args = variant.generics.stream_args();
            quote! { #var_ident #args }
        };

        let if_var_from_enum = quote! {
            if let #enum_ident::#var_ident(__var) = #fn_input {
                ::std::result::Result::Ok(__var)
            }
        };

        (var_ty, if_var_from_enum)
    } else {
        match &variant.fields {
            SaneVarFields::Named(named) => {
                if named.fields.len() != 1 {
                    bail!(named => HELP_CONVERT_1, var_ident => HELP_CONVERT_2);
                }

                let VarFieldNamed {
                    ident: field_ident,
                    ty: var_ty,
                    ..
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

                let VarFieldUnnamed { ty: var_ty, .. } = &unnamed.fields[0];

                let if_var_from_enum = quote! {
                    if let #enum_ident::#var_ident(__var) = #fn_input {
                        ::std::result::Result::Ok(__var)
                    }
                };

                (quote! { #var_ty }, if_var_from_enum)
            }
            SaneVarFields::Unit => {
                bail!(variant.fields => HELP_CONVERT_1, var_ident => HELP_CONVERT_2)
            }
        }
    };

    Ok(quote! {
        impl #gen_params ::std::convert::TryFrom<#enum_ty> for #var_ty #where_clause {
            type Error = #enum_ty;

            fn try_from(#fn_input: #enum_ty) -> ::std::result::Result<Self, Self::Error> {
                #if_var_from_enum
                else { ::std::result::Result::Err(#fn_input) }
            }
        }

        impl #gen_lf_params ::std::convert::TryFrom<&#lf #enum_ty> for &#lf #var_ty #where_clause {
            type Error = ();

            fn try_from(#fn_input: &#lf #enum_ty) -> ::std::result::Result<Self, Self::Error> {
                #if_var_from_enum
                else { ::std::result::Result::Err(()) }
            }
        }

        impl #gen_lf_params ::std::convert::TryFrom<&#lf mut #enum_ty> for &#lf mut #var_ty #where_clause {
            type Error = ();

            fn try_from(#fn_input: &#lf mut #enum_ty) -> ::std::result::Result<Self, Self::Error> {
                #if_var_from_enum
                else { ::std::result::Result::Err(()) }
            }
        }

        impl #gen_params ::spire_enum::prelude::FromEnum<#enum_ty> for #var_ty #where_clause {
            fn from_enum(#fn_input: #enum_ty) -> ::std::result::Result<Self, #enum_ty> {
                #if_var_from_enum
                else { ::std::result::Result::Err(#fn_input) }
            }
        }

        impl #gen_lf_params ::spire_enum::prelude::FromEnumRef<#enum_ty> for #var_ty #where_clause {
            fn from_enum_ref<'__ref>(#fn_input: &'__ref #enum_ty) -> ::std::option::Option<&'__ref Self> {
                if let #enum_ident::#var_ident(__var) = #fn_input {
                    ::std::option::Option::Some(__var)
                } else {
                    ::std::option::Option::None
                }
            }
        }

        impl #gen_lf_params ::spire_enum::prelude::FromEnumMut<#enum_ty> for #var_ty #where_clause {
            fn from_enum_mut<'__ref>(#fn_input: &'__ref mut #enum_ty) -> ::std::option::Option<&'__ref mut Self> {
                if let #enum_ident::#var_ident(__var) = #fn_input {
                    ::std::option::Option::Some(__var)
                } else {
                    ::std::option::Option::None
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
        ident: enum_ident,
        generics: enum_generics,
        ty: enum_ty,
        ..
    } = enum_def;

    let (enum_generics, where_clause) = enum_generics.as_pair();

    let fn_input = Ident::new("_value", Span::call_site());
    let var_ident = &variant.ident;

    let will_type_be_generated = settings.extract_variants.is_some() && variant.allow_extract();

    let (var_ty, ret_enum_from_var) = if will_type_be_generated {
        let var_ty = {
            let args = variant.generics.stream_args();
            quote! { #var_ident #args }
        };

        let ret_enum_from_var = quote! {
            #enum_ident::#var_ident(#fn_input)
        };

        (var_ty, ret_enum_from_var)
    } else {
        match &variant.fields {
            SaneVarFields::Named(named) => {
                if named.fields.len() != 1 {
                    bail!(named => HELP_CONVERT_1, var_ident => HELP_CONVERT_2);
                }

                let VarFieldNamed {
                    ident: field_ident,
                    ty: var_ty,
                    ..
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

                let VarFieldUnnamed { ty: var_ty, .. } = &unnamed.fields[0];

                let ret_enum_from_var = quote! {
                    #enum_ident::#var_ident(#fn_input)
                };

                (quote! { #var_ty }, ret_enum_from_var)
            }
            SaneVarFields::Unit => {
                bail!(variant.fields => HELP_CONVERT_1, var_ident => HELP_CONVERT_2)
            }
        }
    };

    Ok(quote! {
        impl #enum_generics From::<#var_ty> for #enum_ty #where_clause {
            fn from(#fn_input: #var_ty) -> Self {
                #ret_enum_from_var
            }
        }
    })
}

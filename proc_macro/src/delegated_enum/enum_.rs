use super::*;

pub struct SaneEnum {
    pub attrs: Any<Attribute<SynMeta>>,
    pub vis: Visibility,
    pub enum_token: Token![enum],
    pub ident: Ident,
    pub generics: Optional<SaneGenerics>,
    pub _brace: syn::token::Brace,
    pub variants: Vec<SaneVar>,
    pub ty: Box<Type>,
}

pub(super) fn run(enum_stream: TokenStream1, settings: Settings) -> Result<TokenStream> {
    let input_enum = syn::parse::<Enum<Meta<VarMeta>, Meta<kw_delegator>>>(enum_stream)?;
    let enum_def = sanitize_input(input_enum, &settings)?;

    let mut stream = TokenStream::new();

    if let _Some(SaneSettingExtractVariants {
        kw: _,
        attrs: each_attrs,
        enum_derives,
    }) = &settings.extract_variants
    {
        let derive_list = enum_derives
            .is_some()
            .then(|| gather_enum_derives(&enum_def));

        let enum_derives = derive_list.as_deref().unwrap_or_default();

        for var in enum_def.variants.iter().filter(|var| var.allow_extract()) {
            stream.extend(generate_variant_type_definition(
                var,
                &enum_def,
                each_attrs,
                enum_derives,
            ));
        }
    }

    let vars_allow_conversions = enum_def
        .variants
        .iter()
        .filter(|var| var.allow_generate_conversions());

    let should_impl_enum_into_vars = settings.should_impl_enum_try_into_variants();

    if should_impl_enum_into_vars {
        let gen_params = enum_def.generics.stream_params();
        let where_clause = enum_def.generics.as_pair().1;
        let enum_ty = &enum_def.ty;
        stream.extend(quote! {
            impl #gen_params ::spire_enum::prelude::EnumExtensions for #enum_ty #where_clause {}
        });
    }

    match (should_impl_enum_into_vars, settings.should_impl_variants_into_enum()) {
        (true, true) => {
            for var in vars_allow_conversions {
                stream.extend(generate_variant_try_from_enum(var, &enum_def, &settings)?);
                stream.extend(generate_enum_from_variant(var, &enum_def, &settings)?);
            }
        }
        (true, false) => {
            for var in vars_allow_conversions {
                stream.extend(generate_variant_try_from_enum(var, &enum_def, &settings)?);
            }
        }
        (false, true) => {
            for var in vars_allow_conversions {
                stream.extend(generate_enum_from_variant(var, &enum_def, &settings)?);
            }
        }
        (false, false) => {}
    }

    stream.extend(generate_enum_type(&enum_def, &settings));
    stream.extend(generate_delegate_macro(&enum_def, &settings));

    Ok(stream)
}

fn sanitize_input(
    input: Enum<Meta<VarMeta>, Meta<kw_delegator>>,
    settings: &Settings,
) -> Result<SaneEnum> {
    let Enum {
        attrs,
        vis,
        enum_token,
        ident,
        generics,
        where_clause,
        variants,
    } = input;

    let generics = sanitize_generics(generics, where_clause)?;

    let (brace, variants) = variants.into_parts();

    let variants = variants
        .inner
        .into_iter()
        .map(|var| sanitize_variant(var, settings, &generics))
        .try_collect::<_, Vec<_>, _>()?;

    let ty: Box<Type> = {
        let enum_gen_args = generics.stream_args();

        (|| Ok(try_parse_quote!(#ident #enum_gen_args)))().map_err(|mut err: Error| {
			let msg = Error::new(
				ident.span(),
				"parse_quote! failed to convert tokens into a syn::Type, we tried to merge this ident..",
			);
			err.combine(msg);

			let msg = Error::new(enum_gen_args.span(), "..with these generics");
			err.combine(msg);

			err
		})?
    };

    Ok(SaneEnum {
        attrs,
        vis,
        enum_token,
        ident,
        generics,
        _brace: brace,
        variants,
        ty,
    })
}

fn generate_enum_type(enum_def: &SaneEnum, settings: &Settings) -> Result<TokenStream> {
    let SaneEnum {
        attrs,
        vis,
        enum_token,
        ident,
        generics,
        _brace: _,
        variants,
        ty: _,
    } = enum_def;

    let (generics, where_clause) = generics.as_pair();
    let variants_definitions = variants
        .iter()
        .map(|var| generate_enum_variant_definition(var, settings))
        .try_collect::<_, Vec<_>, _>()?;

    Ok(quote! {
        #attrs
        #vis #enum_token #ident #generics #where_clause {
            #( #variants_definitions ),*
        }
    })
}

fn generate_delegate_macro(enum_def: &SaneEnum, settings: &Settings) -> Result<TokenStream> {
    let enum_ident = &enum_def.ident;

    let (cases_closure, cases_tokens) = enum_def
        .variants
        .iter()
        .map(|variant| {
            let will_variant_be_generated =
                settings.extract_variants.is_some() && variant.allow_extract();

            match &variant.explicit_delegator {
                _Some(ExplicitDelegator::Expr(_, expr)) => {
                    Ok(handle_delegator_closure(enum_def, variant, will_variant_be_generated, expr))
                }
                _None => {
                    match &variant.fields {
                        SaneVarFields::Named(SaneVarFieldsNamed {
                            delegator: Some((_, field_ident)),
                            ..
                        }) => {
                            Ok(handle_delegator_field_named(
                                enum_def,
                                variant,
                                field_ident,
                                will_variant_be_generated,
                            ))
                        }
                        SaneVarFields::Unnamed(SaneVarFieldsUnnamed {
                            delegator: Some((_, field_idx)),
                            ..
                        }) => {
                            Ok(handle_delegator_field_unnamed(
                                enum_def,
                                variant,
                                *field_idx,
                                will_variant_be_generated,
                            ))
                        }
                        _ => {
                            handle_no_explicit_delegator(
                                enum_def,
                                variant,
                                will_variant_be_generated,
                            )
                        }
                    }
                }
            }
        })
        .try_collect::<_, Vec<_>, _>()?
        .into_iter()
        .unzip::<_, _, Vec<_>, Vec<_>>();

    let macro_ident = delegate_macro_ident(enum_ident);

    let macro_docs = docs_tokens(format!(
		"\n\
		This macro was generated by an invocation of [`delegated_enum`](spire_enum_macros::delegated_enum).\n\
		\n\
		Performs delegation of the enum by applying the same expression to all variants of the enum.\n\
        \n\
        This macro is used in code generated by invocations of [`delegate_impl`](spire_enum_macros::delegate_impl), \
        though nothing stops you from using it manually.\n\
        \n\
        ## This macro accepts 2 different syntaxes:\n\
        \n\
        ```rust ignore\n\
        // Simple\n\
        `{macro_ident}!{{ enum_variable.method_name(comma,separated,method,args).continue_with_any_rust_expression() }}`\n\
        \n\
        // Closure \n\
        `{macro_ident}!{{ enum_variable => |arg| any_rust_expression_that_uses_arg_parameter() }}`\n\
        ```\n\
		\n\
		## Example:\n\
		```rust ignore\n\
        impl Clone for {enum_ident} {{
    fn clone(&self) -> Self {{
        {macro_ident}!{{ self.clone() }}
    }}\n\
        }}\n\
		```\n\
		"
	));

    Ok(quote! {
        #macro_docs
        macro_rules! #macro_ident {
            ( $_Self:expr => |$arg:ident| $($Rest: tt)* ) => {
                match $_Self {
                    #(#cases_closure)*
                }
            };

            ( $_Self:tt $($Rest: tt)* ) => {
                match $_Self {
                    #(#cases_tokens)*
                }
            };
        }

        pub(crate) use #macro_ident;
    })
}

fn handle_delegator_closure(
    enum_def: &SaneEnum,
    variant: &SaneVar,
    will_variant_be_generated: bool,
    expr: &Paren<ExprClosure>,
) -> (TokenStream, TokenStream) {
    let enum_ident = &enum_def.ident;
    let var_ident = &variant.ident;

    if will_variant_be_generated {
        let closure = quote! {
            #enum_ident::#var_ident(__var) => {
                let __f = #expr;
                let $arg = __f(__var);
                $($Rest)*
            }
        };

        let tokens = quote! {
            #enum_ident::#var_ident(__var) => {
                let __f = #expr;
                let __res = __f(__var);
                __res $($Rest)*
            }
        };

        (closure, tokens)
    } else {
        match &variant.fields {
            SaneVarFields::Named(SaneVarFieldsNamed { fields, .. }) => {
                let field_idents = fields
                    .iter()
                    .take(expr.inputs.len())
                    .map(|field| &field.ident)
                    .collect::<Vec<_>>();

                let closure = quote! {
                    #enum_ident::#var_ident { #(#field_idents),* , .. } => {
                        let __f = #expr;
                        let $arg = __f(#(#field_idents),*);
                        $($Rest)*
                    }
                };

                let tokens = quote! {
                    #enum_ident::#var_ident { #(#field_idents),* , .. } => {
                        let __f = #expr;
                        let __res = __f(#(#field_idents),*);
                        __res $($Rest)*
                    }
                };

                (closure, tokens)
            }
            SaneVarFields::Unnamed(SaneVarFieldsUnnamed { fields, .. }) => {
                let field_idents = fields
                    .iter()
                    .take(expr.inputs.len())
                    .enumerate()
                    .map(|(idx, _)| Ident::new(&format!("__{idx}"), Span::call_site()))
                    .collect::<Vec<_>>();

                let closure = quote! {
                    #enum_ident::#var_ident(#(#field_idents),* , ..) => {
                        let __f = #expr;
                        let $arg = __f(#(#field_idents),*);
                        $($Rest)*
                    }
                };

                let tokens = quote! {
                    #enum_ident::#var_ident(#(#field_idents),* , ..) => {
                        let __f = #expr;
                        let __res = __f(#(#field_idents),*);
                        __res $($Rest)*
                    }
                };

                (closure, tokens)
            }
            SaneVarFields::Unit => {
                let closure = quote! {
                    #enum_ident::#var_ident => {
                        let __f = #expr;
                        let $arg = __f();
                        $($Rest)*
                    }
                };

                let tokens = quote! {
                    #enum_ident::#var_ident => {
                        let __f = #expr;
                        let __res = __f();
                        __res $($Rest)*
                    }
                };

                (closure, tokens)
            }
        }
    }
}

const HELP_MISSING_DELEGATOR: &str = r##"
Expected variant to have a delegator.
Help: The delegator is an expression that defines which variable is going
to call methods when matched through the enum.

Example:
```rust
// Given the enum:
enum DelegatedEnum {
	Unnamed(Vec<i32>),
	Named { field: i64 },
	Unit,
}

// Imagine we want to delegate the method `foo() -> String`
impl DelegatedEnum {
	pub fn foo(&self) -> String {
		match self {
			// Delegator: `vec`
			Self::Unnamed(vec) => vec.foo(),
			// Delegator: `field`
			Self::Named { field } => field.foo(),
			// No Delegator: compiler error
			Self::Unit => {}
		}
	}
}
```

Help: If you want to delegate to a specific field, 
you can add the attribute `#[delegate_via_field(field_name)]` to the variant:
enum DelegatedEnum {
	Unnamed(Vec<i32>),
	#[delegate_via_field(field_two)]
	Named { field_one: i64, field_two: String },
	Unit,
}

Help: If you want to delegate to a more complex expression, use #[delegate_via(|| {block})]:
enum DelegatedEnum {
	Unnamed(Vec<i32>),
	Named { field: i64 },
	#[delegate_via(|| 5i64)]
	Unit,
}

Help: If no explicit delegator is specified, the first field of the variant will be inferred
as the delegator, Unit variants will cause a compiler error.
"##;

fn handle_no_explicit_delegator(
    enum_def: &SaneEnum,
    variant: &SaneVar,
    will_variant_be_generated: bool,
) -> Result<(TokenStream, TokenStream)> {
    if will_variant_be_generated {
        let enum_ident = &enum_def.ident;
        let var_ident = &variant.ident;

        let closure = quote! {
            #enum_ident::#var_ident($arg) => { $($Rest)* }
        };

        let tokens = quote! {
            #enum_ident::#var_ident(__var) => {
                __var $($Rest)*
            }
        };

        Ok((closure, tokens))
    } else {
        match &variant.fields {
            SaneVarFields::Named(SaneVarFieldsNamed { fields, .. }) => {
                if let Some(VarFieldNamed {
                    ident: field_ident, ..
                }) = fields.first()
                {
                    Ok(handle_delegator_field_named(enum_def, variant, field_ident, false))
                } else {
                    bail!(variant.ident => HELP_MISSING_DELEGATOR)
                }
            }
            SaneVarFields::Unnamed(SaneVarFieldsUnnamed { fields, .. }) => {
                if !fields.is_empty() {
                    Ok(handle_delegator_field_unnamed(enum_def, variant, 0, false))
                } else {
                    bail!(variant.ident => HELP_MISSING_DELEGATOR)
                }
            }
            SaneVarFields::Unit => bail!(variant.ident => HELP_MISSING_DELEGATOR),
        }
    }
}

fn handle_delegator_field_unnamed(
    enum_def: &SaneEnum,
    variant: &SaneVar,
    field_idx: usize,
    will_variant_be_generated: bool,
) -> (TokenStream, TokenStream) {
    let enum_ident = &enum_def.ident;
    let var_ident = &variant.ident;

    let mut fields = Punctuated::<Token![_], Token![,]>::new();
    for _ in 0..field_idx {
        fields.push(Default::default());
    }

    if !fields.is_empty() && !fields.trailing_punct() {
        fields.push_punct(Default::default());
    }

    if will_variant_be_generated {
        (
            quote!(#enum_ident::#var_ident(#var_ident(#fields $arg, ..), ..) => { $($Rest)* } ),
            quote!(#enum_ident::#var_ident(#var_ident(#fields __var, ..), ..) => { __var $($Rest)* }),
        )
    } else {
        (
            quote!(#enum_ident::#var_ident(#fields $arg, ..) => { $($Rest)* } ),
            quote!(#enum_ident::#var_ident(#fields __var, ..) => { __var $($Rest)* }),
        )
    }
}

fn handle_delegator_field_named(
    enum_def: &SaneEnum,
    variant: &SaneVar,
    field_ident: &Ident,
    will_variant_be_generated: bool,
) -> (TokenStream, TokenStream) {
    let enum_ident = &enum_def.ident;
    let var_ident = &variant.ident;

    if will_variant_be_generated {
        (
            quote!(#enum_ident::#var_ident(#var_ident { #field_ident: $arg, .. }) => { $($Rest)* } ),
            quote!(#enum_ident::#var_ident(#var_ident { #field_ident, .. }) => { #field_ident $($Rest)* } ),
        )
    } else {
        (
            quote! { #enum_ident::#var_ident { #field_ident: $arg, .. } => { $($Rest)* } },
            quote! { #enum_ident::#var_ident { #field_ident, .. } => { #field_ident $($Rest)* } },
        )
    }
}

fn gather_enum_derives(enum_def: &SaneEnum) -> Vec<&Attribute<SynMeta>> {
    enum_def
        .attrs
        .iter()
        .filter(|attr| {
            if let SynMeta::List(meta_list) = &*attr.inner {
                meta_list.path.is_ident("derive")
            } else {
                false
            }
        })
        .collect()
}

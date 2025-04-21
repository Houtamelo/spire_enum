use super::*;

pub struct SaneEnum {
	pub attrs: Any<InputAttribute>,
	pub vis: Visibility,
	pub enum_token: Token![enum],
	pub ident: Ident,
	pub generics: Optional<SaneGenerics>,
	pub _brace: syn::token::Brace,
	pub variants: Vec<SaneVariant>,
	pub ty: Box<Type>,
}

pub(super) fn run(enum_stream: TokenStream1, settings: Settings) -> Result<TokenStream> {
	let input_enum = syn::parse::<InputEnum>(enum_stream)?;
	let enum_def = sanitize_input(input_enum, &settings)?;

	let mut stream = TokenStream::new();

	if let Some(SaneGenerateVariants {
		kw: _,
		attrs: each_attrs,
	}) = &settings.generate_variants
	{
		for var in enum_def
			.variants
			.iter()
			.filter(|var| var.allow_generate_type())
		{
			stream.extend(generate_variant_type_definition(var, &enum_def, each_attrs));
		}
	}

	let vars_allow_conversions = enum_def
		.variants
		.iter()
		.filter(|var| var.allow_generate_conversions());

	match (settings.should_impl_enum_try_into_variants(), settings.should_impl_variants_into_enum())
	{
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

#[derive(Parse, ToTokens)]
struct InputEnum {
	attrs: Any<InputAttribute>,
	vis: Visibility,
	enum_token: Token![enum],
	ident: Ident,
	generics: Optional<InputGenerics>,
	where_clause: Optional<WhereClause>,
	variants: Brace<Punctuated<InputVariant, Token![,]>>,
}

fn sanitize_input(input: InputEnum, settings: &Settings) -> Result<SaneEnum> {
	let InputEnum {
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
		.into_iter()
		.map(|var| sanitize_variant(var, settings, &generics))
		.try_collect::<_, Vec<_>, _>()?;

	let ty: Box<Type> = {
		let enum_generics = generics.to_tokens_without_bounds()?;

		(|| Ok(try_parse_quote!(#ident #enum_generics)))().map_err(|mut err: Error| {
			let msg = Error::new(
				ident.span(),
				"parse_quote! failed to convert tokens into a syn::Type, we tried to merge this ident..",
			);
			err.combine(msg);

			let msg = Error::new(enum_generics.span(), "..with these generics");
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
				settings.generate_variants.is_some() && variant.allow_generate_type();

			match &variant.explicit_delegator {
				_Some(ExplicitDelegator::Expr(_, expr)) => {
					Ok(handle_delegator_closure(enum_def, variant, will_variant_be_generated, expr))
				}
				_None => {
					match &variant.fields {
						SaneVariantFields::Named(SaneVariantFieldsNamed {
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
						SaneVariantFields::Unnamed(SaneVariantFieldsUnnamed {
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

	Ok(quote! {
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
	variant: &SaneVariant,
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
			SaneVariantFields::Named(SaneVariantFieldsNamed { fields, .. }) => {
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
			SaneVariantFields::Unnamed(SaneVariantFieldsUnnamed { fields, .. }) => {
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
			SaneVariantFields::Unit => {
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
	variant: &SaneVariant,
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
			SaneVariantFields::Named(SaneVariantFieldsNamed { fields, .. }) => {
				if let Some(VariantFieldNamed {
					ident: field_ident, ..
				}) = fields.first()
				{
					Ok(handle_delegator_field_named(enum_def, variant, field_ident, false))
				} else {
					bail!(variant.ident => HELP_MISSING_DELEGATOR)
				}
			}
			SaneVariantFields::Unnamed(SaneVariantFieldsUnnamed { fields, .. }) => {
				if !fields.is_empty() {
					Ok(handle_delegator_field_unnamed(enum_def, variant, 0, false))
				} else {
					bail!(variant.ident => HELP_MISSING_DELEGATOR)
				}
			}
			SaneVariantFields::Unit => bail!(variant.ident => HELP_MISSING_DELEGATOR),
		}
	}
}

fn handle_delegator_field_unnamed(
	enum_def: &SaneEnum,
	variant: &SaneVariant,
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
	variant: &SaneVariant,
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

use super::*;

const HELP_CONVERT_1: &str = "Expected variant to have exactly one field. \n\
		Help: For conversions to be implemented, the variant must satisfy one of:\n\
		- Have exactly one field (and that field's type will be the one implementing conversions).\n\
		- Have the setting \"generate type\" turned on (and the generated variant's type \
		will be the one implementing conversions).";

const HELP_CONVERT_2: &str = "Help: If you do not want to generate conversions, \
		add the attribute #[no_convert] to this variant";

pub fn generate_variant_try_from_enum(
	variant: &SaneVariant,
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

	let var_ident = &variant.ident;
	let fn_input = Ident::new("__input", Span::call_site());

	let will_type_be_generated =
		settings.generate_variants.is_some() && variant.allow_generate_type();

	let (var_ty, if_var_from_enum) = if will_type_be_generated {
		let var_ty = {
			let generics = variant.generics.to_tokens_without_bounds()?;
			quote! { #var_ident #generics }
		};

		let if_var_from_enum = quote! {
			if let #enum_ident::#var_ident(__var) = #fn_input {
				Ok(__var)
			}
		};

		(var_ty, if_var_from_enum)
	} else {
		match &variant.fields {
			SaneVariantFields::Named(named) => {
				if named.fields.len() != 1 {
					bail!(named => HELP_CONVERT_1, var_ident => HELP_CONVERT_2);
				}

				let VariantFieldNamed {
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
			SaneVariantFields::Unnamed(unnamed) => {
				if unnamed.fields.len() != 1 {
					bail!(unnamed => HELP_CONVERT_1, var_ident => HELP_CONVERT_2);
				}

				let VariantFieldUnnamed { ty: var_ty, .. } = &unnamed.fields[0];

				let if_var_from_enum = quote! {
					if let #enum_ident::#var_ident(__var) = #fn_input {
						Ok(__var)
					}
				};

				(quote! { #var_ty }, if_var_from_enum)
			}
			SaneVariantFields::Unit => {
				bail!(variant.fields => HELP_CONVERT_1, var_ident => HELP_CONVERT_2)
			}
		}
	};

	Ok(quote! {
		impl #enum_generics TryFrom::<#enum_ty> for #var_ty #where_clause {
			type Error = #enum_ty;

			fn try_from(#fn_input: #enum_ty) -> Result<Self, Self::Error> {
				#if_var_from_enum
				else { Err(#fn_input) }
			}
		}
	})
}

pub fn generate_enum_from_variant(
	variant: &SaneVariant,
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

	let fn_input = Ident::new("__input", Span::call_site());
	let var_ident = &variant.ident;

	let will_type_be_generated =
		settings.generate_variants.is_some() && variant.allow_generate_type();

	let (var_ty, ret_enum_from_var) = if will_type_be_generated {
		let var_ty = {
			let generics = variant.generics.to_tokens_without_bounds()?;
			quote! { #var_ident #generics }
		};

		let ret_enum_from_var = quote! {
			#enum_ident::#var_ident(#fn_input)
		};

		(var_ty, ret_enum_from_var)
	} else {
		match &variant.fields {
			SaneVariantFields::Named(named) => {
				if named.fields.len() != 1 {
					bail!(named => HELP_CONVERT_1, var_ident => HELP_CONVERT_2);
				}

				let VariantFieldNamed {
					ident: field_ident,
					ty: var_ty,
					..
				} = &named.fields[0];

				let ret_enum_from_var = quote! {
					#enum_ident::#var_ident { #field_ident: #fn_input }
				};

				(quote! { #var_ty }, ret_enum_from_var)
			}
			SaneVariantFields::Unnamed(unnamed) => {
				if unnamed.fields.len() != 1 {
					bail!(unnamed => HELP_CONVERT_1, var_ident => HELP_CONVERT_2);
				}

				let VariantFieldUnnamed { ty: var_ty, .. } = &unnamed.fields[0];

				let ret_enum_from_var = quote! {
					#enum_ident::#var_ident(#fn_input)
				};

				(quote! { #var_ty }, ret_enum_from_var)
			}
			SaneVariantFields::Unit => {
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

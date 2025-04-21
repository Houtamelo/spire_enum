pub mod fields;
mod sanitize;

mod var_kw {
	use super::*;
	custom_keyword!(dont_generate_type);
	custom_keyword!(dont_impl_conversions);
	custom_keyword!(delegate_via);
}

use fields::*;
pub use sanitize::{ExplicitDelegator, SaneVariant, sanitize_variant};

use super::*;

#[derive(Parse, ToTokens)]
pub struct InputVariant {
	attrs: Any<InputAttribute<VariantMeta>>,
	ident: Ident,
	fields: VariantFields<VariantFieldMeta>,
	discriminant: Optional<InputVariantDiscriminant>,
}

#[derive(Clone, Parse, ToTokens)]
pub struct InputVariantDiscriminant {
	pub eq_token: Token![=],
	pub expr: Expr,
}

#[derive(Parse, ToTokens)]
enum VariantMeta {
	NoVarType(var_kw::dont_generate_type),
	NoConversions(var_kw::dont_impl_conversions),
	DelegateVia(var_kw::delegate_via, Paren<ExprClosure>),
	Syn(Meta),
}

pub fn generate_variant_type_definition(
	variant: &SaneVariant,
	enum_def: &SaneEnum,
	extra_attrs: &[Meta],
) -> TokenStream {
	let attrs = &variant.attrs.syn_attrs;

	let vis = &enum_def.vis;
	let ident = &variant.ident;

	let (generics, where_clause) = variant.generics.as_pair();

	let fields_and_where_clause = match &variant.fields {
		SaneVariantFields::Named(named) => {
			let fields = named.fields.iter().map(
				|VariantFieldNamed {
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
		SaneVariantFields::Unnamed(unnamed) => {
			let fields = unnamed
				.fields
				.iter()
				.map(|VariantFieldUnnamed { attrs, ty }| {
					quote! { #attrs pub #ty }
				});

			quote! { ( #(#fields),* ) #where_clause ; }
		}
		SaneVariantFields::Unit => quote! { #where_clause ; },
	};

	quote! {
		#( #[#attrs] )*
		#( #[#extra_attrs] )*
		#vis struct #ident #generics #fields_and_where_clause
	}
}

/*
impl ToTokens for SaneVariantFieldsNamed {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let fields_tt = self.fields.iter().map(
			|VariantFieldNamed {
				 attrs,
				 ident,
				 colon_token,
				 ty,
			 }| {
				quote! { #attrs pub #ident #colon_token #ty }
			},
		);

		tokens.append_all(quote!( { #( #fields_tt ),* } ));
	}
}

impl ToTokens for SaneVariantFieldsUnnamed {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		let fields_tt = self.fields.iter().map(|VariantFieldUnnamed { attrs, ty }| {
			quote! { #attrs pub #ty }
		});

		tokens.append_all(quote! { ( #( #fields_tt ),* ) });
	}
}

*/

pub fn generate_enum_variant_definition(
	variant: &SaneVariant,
	settings: &Settings,
) -> Result<TokenStream> {
	let attrs = &variant.attrs.syn_attrs;
	let var_ident = &variant.ident;
	let discriminant = &variant.discriminant;

	let will_type_be_generated =
		settings.generate_variants.is_some() && variant.allow_generate_type();

	if will_type_be_generated {
		let generics = variant.generics.to_tokens_without_bounds()?;

		let var_ty: Type = {
			(|| Ok(try_parse_quote!(#var_ident #generics)))().map_err(|mut err: Error| {
				let msg = Error::new(
					var_ident.span(),
					"parse_quote! failed to create a syn::Type, we tried to merge this ident..",
				);
				err.combine(msg);

				let msg = Error::new(generics.span(), "..with these generics");
				err.combine(msg);

				err
			})?
		};

		Ok(quote! {
			#( #[#attrs] )*
			#var_ident ( #var_ty )
		})
	} else {
		let fields = &variant.fields;

		Ok(quote! {
			#( #[#attrs] )*
			#var_ident #fields #discriminant
		})
	}
}

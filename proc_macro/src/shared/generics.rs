use super::*;

#[derive(ToTokens, Clone)]
pub struct InputGenerics {
	pub _left_angle_bracket: Token![<],
	pub params: syn::punctuated::Punctuated<GenericParam, Token![,]>,
	pub _right_angle_bracket: Token![>],
}

#[derive(Parse, ToTokens)]
pub enum InputGenericParam {
	Lifetime(Box<InputGenericParamLifetime>),
	Type(Box<InputGenericParamType>),
	Const(Box<InputGenericParamConst>),
}

impl IntoSyn for InputGenericParam {
	type Target = GenericParam;

	fn into_syn(self) -> GenericParam {
		match self {
			InputGenericParam::Lifetime(lf) => lf.into_syn().into(),
			InputGenericParam::Type(ty) => ty.into_syn().into(),
			InputGenericParam::Const(cn) => cn.into_syn().into(),
		}
	}
}

#[derive(Clone, Parse, ToTokens)]
pub struct InputGenericParamLifetime {
	pub attrs: Any<InputAttribute>,
	pub lifetime: Lifetime,
	pub bounds: InputGenericParamLifetimeBounds,
}

#[derive(Clone, Parse, ToTokens)]
pub enum InputGenericParamLifetimeBounds {
	Some(Token![:], Separated<Lifetime, Token![+]>),
	None,
}

impl IntoSyn for InputGenericParamLifetimeBounds {
	type Target = (Option<Token![:]>, syn::punctuated::Punctuated<Lifetime, Token![+]>);

	fn into_syn(self) -> Self::Target {
		match self {
			InputGenericParamLifetimeBounds::Some(opt, bounds) => (Some(opt), bounds.into_inner()),
			InputGenericParamLifetimeBounds::None => (None, Default::default()),
		}
	}
}

impl IntoSyn for InputGenericParamLifetime {
	type Target = LifetimeParam;

	fn into_syn(self) -> LifetimeParam {
		let (colon_token, bounds) = self.bounds.into_syn();

		LifetimeParam {
			attrs: self.attrs.into_syn(),
			lifetime: self.lifetime,
			colon_token,
			bounds,
		}
	}
}

#[derive(Clone, Parse, ToTokens)]
pub struct InputGenericParamType {
	pub attrs: Any<InputAttribute>,
	pub ident: Ident,
	pub colon_token: Option<Token![:]>,
	pub bounds: Punctuated<TypeParamBound, Token![+]>,
	pub eq_token: Option<Token![=]>,
	pub default: Optional<Type>,
}

impl IntoSyn for InputGenericParamType {
	type Target = TypeParam;

	fn into_syn(self) -> TypeParam {
		TypeParam {
			attrs: self.attrs.into_syn(),
			ident: self.ident,
			colon_token: self.colon_token,
			bounds: self.bounds.into_inner(),
			eq_token: self.eq_token,
			default: self.default.into_syn(),
		}
	}
}

#[derive(Clone, Parse, ToTokens)]
pub struct InputGenericParamConst {
	pub attrs: Any<InputAttribute>,
	pub const_token: Token![const],
	pub ident: Ident,
	pub colon_token: Token![:],
	pub ty: Type,
	pub eq_token: Option<Token![=]>,
	pub default: Optional<Expr>,
}

impl IntoSyn for InputGenericParamConst {
	type Target = ConstParam;

	fn into_syn(self) -> ConstParam {
		ConstParam {
			attrs: self.attrs.into_syn(),
			const_token: self.const_token,
			ident: self.ident,
			colon_token: self.colon_token,
			ty: self.ty,
			eq_token: self.eq_token,
			default: self.default.into_syn(),
		}
	}
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
			_left_angle_bracket,
			params: params.into_iter().collect(),
			_right_angle_bracket,
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
		(_Some(generics), where_clause) => {
			Ok(_Some(SaneGenerics {
				input: generics,
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
	pub fn into_syn(self) -> syn::Generics {
		syn::Generics {
			lt_token: Some(self.input._left_angle_bracket),
			params: self.input.params,
			gt_token: Some(self.input._right_angle_bracket),
			where_clause: self.where_clause.into_syn(),
		}
	}

	pub fn to_tokens_without_bounds(&self) -> Result<TokenStream> {
		let InputGenerics {
			_left_angle_bracket,
			params,
			_right_angle_bracket,
		} = &self.input;

		let params_tt = params.iter().map(|p| {
			match p {
				GenericParam::Lifetime(lf) => lf.lifetime.to_token_stream(),
				GenericParam::Type(ty) => ty.ident.to_token_stream(),
				GenericParam::Const(cn) => cn.ident.to_token_stream(),
			}
		});

		Ok(try_parse_quote!(#_left_angle_bracket #(#params_tt),* #_right_angle_bracket))
	}
}

impl Optional<SaneGenerics> {
	#[allow(unused)]
	pub fn into_syn(self) -> syn::Generics {
		if let _Some(generics) = self {
			generics.into_syn()
		} else {
			Default::default()
		}
	}

	pub fn to_tokens_without_bounds(&self) -> Result<TokenStream> {
		if let _Some(generics) = self {
			generics.to_tokens_without_bounds()
		} else {
			Ok(Default::default())
		}
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

#[allow(unused)]
pub trait OptionalGenericsImpl {
	fn into_syn(self) -> syn::Generics;
	fn to_tokens_without_bounds(&self) -> Result<TokenStream>;
}

impl OptionalGenericsImpl for Option<SaneGenerics> {
	fn into_syn(self) -> syn::Generics {
		if let Some(generics) = self {
			generics.into_syn()
		} else {
			Default::default()
		}
	}

	fn to_tokens_without_bounds(&self) -> Result<TokenStream> {
		if let Some(generics) = self {
			generics.to_tokens_without_bounds()
		} else {
			Ok(Default::default())
		}
	}
}

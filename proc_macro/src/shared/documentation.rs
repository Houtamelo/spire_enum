use super::*;

pub fn docs_tokens(string: impl AsRef<str>) -> TokenStream {
	let str = string.as_ref();
	quote! { #[doc = #str] }
}

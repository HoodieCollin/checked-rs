use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

use super::kw;

#[derive(Clone)]
pub enum UpperOrMax {
    Upper(kw::upper),
    Max(kw::max),
}

impl Parse for UpperOrMax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::upper) {
            Ok(Self::Upper(input.parse()?))
        } else if input.peek(kw::max) {
            Ok(Self::Max(input.parse()?))
        } else {
            Err(input.error("expected `upper` or `max`"))
        }
    }
}

impl ToTokens for UpperOrMax {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Upper(kw) => kw.to_tokens(tokens),
            Self::Max(kw) => kw.to_tokens(tokens),
        }
    }
}

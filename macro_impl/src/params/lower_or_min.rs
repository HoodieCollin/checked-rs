use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

use super::kw;

#[derive(Clone)]
pub enum LowerOrMin {
    Lower(kw::lower),
    Min(kw::min),
}

impl Parse for LowerOrMin {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::lower) {
            Ok(Self::Lower(input.parse()?))
        } else if input.peek(kw::min) {
            Ok(Self::Min(input.parse()?))
        } else {
            Err(input.error("expected `lower` or `min`"))
        }
    }
}

impl ToTokens for LowerOrMin {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Lower(kw) => kw.to_tokens(tokens),
            Self::Min(kw) => kw.to_tokens(tokens),
        }
    }
}

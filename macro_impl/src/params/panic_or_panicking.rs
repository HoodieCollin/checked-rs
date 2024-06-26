use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

use super::kw;

/// Represents the `Saturate` or `Saturating` keyword.
#[derive(Clone)]
pub enum PanicOrPanicking {
    Panic(kw::Panic),
    Panicking(kw::Panicking),
}

impl Parse for PanicOrPanicking {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::Panic) {
            Ok(Self::Panic(input.parse()?))
        } else if input.peek(kw::Panicking) {
            Ok(Self::Panicking(input.parse()?))
        } else {
            Err(input.error("expected `Panic` or `Panicking`"))
        }
    }
}

impl ToTokens for PanicOrPanicking {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Panic(kw) => kw.to_tokens(tokens),
            Self::Panicking(kw) => kw.to_tokens(tokens),
        }
    }
}

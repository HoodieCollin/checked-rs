use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

use super::kw;

/// Represents the `Saturate` or `Saturating` keyword.
#[derive(Clone)]
pub enum SaturateOrSaturating {
    Saturate(kw::Saturate),
    Saturating(kw::Saturating),
}

impl Parse for SaturateOrSaturating {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::Saturate) {
            Ok(Self::Saturate(input.parse()?))
        } else if input.peek(kw::Saturating) {
            Ok(Self::Saturating(input.parse()?))
        } else {
            Err(input.error("expected `Saturate` or `Saturating`"))
        }
    }
}

impl ToTokens for SaturateOrSaturating {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Saturate(kw) => kw.to_tokens(tokens),
            Self::Saturating(kw) => kw.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_parse, snapshot};

    #[test]
    fn parse_saturate() {
        assert_parse!(SaturateOrSaturating => { Saturate } => { SaturateOrSaturating::Saturate(..) });
    }

    #[test]
    fn parse_saturating() {
        assert_parse!(SaturateOrSaturating => { Saturating } => { SaturateOrSaturating::Saturating(..) });
    }

    #[test]
    fn snapshot_saturate() {
        snapshot!(SaturateOrSaturating => { Saturate });
    }

    #[test]
    fn snapshot_saturating() {
        snapshot!(SaturateOrSaturating => { Saturating });
    }
}

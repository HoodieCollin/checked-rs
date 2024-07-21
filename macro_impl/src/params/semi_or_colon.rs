use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

/// Represents the `MIN` or `MAX` keyword.
#[derive(Clone)]
pub enum SemiOrComma {
    Semi(syn::Token![;]),
    Comma(syn::Token![,]),
}

impl Parse for SemiOrComma {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![;]) {
            Ok(Self::Semi(input.parse()?))
        } else if input.peek(syn::Token![,]) {
            Ok(Self::Comma(input.parse()?))
        } else {
            Err(input.error("expected `;` or `,`"))
        }
    }
}

impl ToTokens for SemiOrComma {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Semi(kw) => kw.to_tokens(tokens),
            Self::Comma(kw) => kw.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_parse, snapshot};

    #[test]
    fn parse_semi() {
        assert_parse!(SemiOrComma => { ; } => { SemiOrComma::Semi(..) });
    }

    #[test]
    fn parse_comma() {
        assert_parse!(SemiOrComma => { , } => { SemiOrComma::Comma(..) });
    }

    #[test]
    fn snapshot_semi() {
        snapshot!(SemiOrComma => { ; });
    }

    #[test]
    fn snapshot_comma() {
        snapshot!(SemiOrComma => { , });
    }
}

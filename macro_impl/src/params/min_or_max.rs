use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

use super::kw;

/// Represents the `MIN` or `MAX` keyword.
#[derive(Clone)]
pub enum MinOrMax {
    Min(kw::MIN),
    Max(kw::MAX),
}

impl Parse for MinOrMax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::MIN) {
            Ok(Self::Min(input.parse()?))
        } else if input.peek(kw::MAX) {
            Ok(Self::Max(input.parse()?))
        } else {
            Err(input.error("expected `MIN` or `MAX`"))
        }
    }
}

impl ToTokens for MinOrMax {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Min(kw) => kw.to_tokens(tokens),
            Self::Max(kw) => kw.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_parse, snapshot};

    #[test]
    fn parse_min() {
        assert_parse!(MinOrMax => { MIN } => { MinOrMax::Min(..) });
    }

    #[test]
    fn parse_max() {
        assert_parse!(MinOrMax => { MAX } => { MinOrMax::Max(..) });
    }

    #[test]
    fn snapshot_min() {
        snapshot!(MinOrMax => { MIN });
    }

    #[test]
    fn snapshot_max() {
        snapshot!(MinOrMax => { MAX });
    }
}

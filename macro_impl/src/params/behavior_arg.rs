use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::Parse;

use super::{kw, PanicOrPanicking, SaturateOrSaturating};

#[derive(Clone)]
pub enum BehaviorArg {
    Saturating(SaturateOrSaturating),
    Panicking(PanicOrPanicking),
}

impl Parse for BehaviorArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::Saturate) || input.peek(kw::Saturating) {
            Ok(Self::Saturating(input.parse()?))
        } else if input.peek(kw::Panic) || input.peek(kw::Panicking) {
            Ok(Self::Panicking(input.parse()?))
        } else {
            Err(input.error("expected `Saturating` or `Panicking`"))
        }
    }
}

impl ToTokens for BehaviorArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::Saturating(..) => quote! {
                Saturating
            },
            Self::Panicking(..) => quote! {
                Panicking
            },
        });
    }
}

impl std::fmt::Debug for BehaviorArg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Saturating(..) => write!(f, "Saturating"),
            Self::Panicking(..) => write!(f, "Panicking"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_parse, snapshot};

    #[test]
    fn parse_saturating() {
        assert_parse!(BehaviorArg => { Saturating } => { BehaviorArg::Saturating(..) });
    }

    #[test]
    fn parse_saturate() {
        assert_parse!(BehaviorArg => { Saturate } => { BehaviorArg::Saturating(..) });
    }

    #[test]
    fn parse_panicking() {
        assert_parse!(BehaviorArg => { Panicking } => { BehaviorArg::Panicking(..) });
    }

    #[test]
    fn parse_panic() {
        assert_parse!(BehaviorArg => { Panic } => { BehaviorArg::Panicking(..) });
    }

    #[test]
    fn to_tokens_saturating() {
        snapshot!(BehaviorArg => { Saturating });
    }

    #[test]
    fn to_tokens_panicking() {
        snapshot!(BehaviorArg => { Panicking });
    }
}

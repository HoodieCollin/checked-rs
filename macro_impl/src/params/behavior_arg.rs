use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::Parse;

use super::{kw, PanicOrPanicking, SaturateOrSaturating};

/// Represents the behavior argument. It can be `Saturating` or `Panicking`.
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

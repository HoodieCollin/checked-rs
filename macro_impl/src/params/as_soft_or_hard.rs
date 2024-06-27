use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

use super::kw;

#[derive(Clone)]
pub enum AsSoftOrHard {
    Soft {
        as_token: syn::Token![as],
        soft: kw::Soft,
    },
    Hard {
        as_token: syn::Token![as],
        hard: kw::Hard,
    },
}

impl Parse for AsSoftOrHard {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let as_token = input.parse()?;
        if input.peek(kw::Soft) {
            Ok(Self::Soft {
                as_token,
                soft: input.parse()?,
            })
        } else if input.peek(kw::Hard) {
            Ok(Self::Hard {
                as_token,
                hard: input.parse()?,
            })
        } else {
            Err(input.error("expected `Soft` or `Hard`"))
        }
    }
}

impl ToTokens for AsSoftOrHard {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Soft { as_token, soft } => {
                as_token.to_tokens(tokens);
                soft.to_tokens(tokens);
            }
            Self::Hard { as_token, hard } => {
                as_token.to_tokens(tokens);
                hard.to_tokens(tokens);
            }
        }
    }
}

impl std::fmt::Debug for AsSoftOrHard {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Soft { .. } => write!(f, "Soft"),
            Self::Hard { .. } => write!(f, "Hard"),
        }
    }
}

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

use crate::params::{kw, NumberArg};

use super::ClampedEnumVariantField;

#[derive(Clone)]
pub struct ClampedEnumVariant {
    pub pound: Option<syn::Token![#]>,
    pub bracket: Option<syn::token::Bracket>,
    pub default_kw: Option<kw::default>,
    pub default_eq: Option<syn::Token![=]>,
    pub default_val: Option<NumberArg>,
    pub ident: syn::Ident,
    pub field: ClampedEnumVariantField,
}

impl Parse for ClampedEnumVariant {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut pound = None;
        let mut bracket = None;
        let mut default_kw = None;
        let mut default_eq = None;
        let mut default_val = None;

        if input.peek(syn::Token![#]) {
            pound = Some(input.parse()?);

            let content;
            bracket = Some(syn::bracketed!(content in input));
            default_kw = Some(content.parse()?);
            default_eq = Some(content.parse()?);
            default_val = Some(content.parse()?);
        }

        Ok(Self {
            pound,
            bracket,
            default_kw,
            default_eq,
            default_val,
            ident: input.parse()?,
            field: input.parse()?,
        })
    }
}

impl ToTokens for ClampedEnumVariant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(pound) = &self.pound {
            pound.to_tokens(tokens);
            self.bracket.as_ref().unwrap().surround(tokens, |tokens| {
                self.default_kw.to_tokens(tokens);
                self.default_eq.to_tokens(tokens);
                self.default_val.to_tokens(tokens);
            });
        }

        self.ident.to_tokens(tokens);
        self.field.to_tokens(tokens);
    }
}

impl std::fmt::Debug for ClampedEnumVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ClampedEnumVariant")
            .field("default_val", &self.default_val)
            .field("ident", &self.ident)
            .field("field", &self.field)
            .finish_non_exhaustive()
    }
}

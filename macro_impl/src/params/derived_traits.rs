use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parenthesized, parse::Parse};

use super::kw;

#[derive(Clone)]
pub struct DerivedTraits {
    pub derive_kw: kw::derive,
    pub paren: syn::token::Paren,
    pub traits: syn::punctuated::Punctuated<syn::TypePath, syn::Token![,]>,
}

impl Parse for DerivedTraits {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let derive_kw = input.parse()?;

        let content;
        parenthesized!(content in input);

        Ok(Self {
            derive_kw,
            paren: syn::token::Paren::default(),
            traits: content.parse_terminated(syn::TypePath::parse, syn::Token![,])?,
        })
    }
}

impl ToTokens for DerivedTraits {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let traits = &self.traits;
        tokens.extend(quote! {
            #[derive(#traits)]
        });
    }
}

impl std::fmt::Debug for DerivedTraits {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let traits = self
            .traits
            .iter()
            .map(|trait_| trait_.to_token_stream())
            .collect::<Vec<_>>();

        f.debug_struct("DerivedTraits")
            .field("traits", &traits)
            .finish_non_exhaustive()
    }
}

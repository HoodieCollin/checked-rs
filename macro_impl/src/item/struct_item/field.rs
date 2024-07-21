use syn::parse::Parse;

use crate::params::NumberArgRange;

pub struct ClampedStructField {
    #[allow(dead_code)]
    paren: syn::token::Paren,
    pub ranges: syn::punctuated::Punctuated<NumberArgRange, syn::Token![,]>,
}

impl Parse for ClampedStructField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let paren = syn::parenthesized!(content in input);

        let ranges = content.parse_terminated(NumberArgRange::parse, syn::Token![,])?;
        Ok(Self { paren, ranges })
    }
}

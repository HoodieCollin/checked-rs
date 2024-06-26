use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

use crate::params::{NumberArg, NumberArgRange, NumberKind, StrictNumberArgRange};

use super::ClampedEnumVariant;

#[derive(Clone)]
pub enum ClampedEnumVariantField {
    Values {
        paren: syn::token::Paren,
        values: syn::punctuated::Punctuated<NumberArg, syn::Token![,]>,
    },
    Ranges {
        paren: syn::token::Paren,
        values: syn::punctuated::Punctuated<NumberArgRange, syn::Token![,]>,
    },
    ClampedEnum {
        bracket: Option<syn::token::Bracket>,
        value_range: Option<StrictNumberArgRange>,
        brace: syn::token::Brace,
        variants: syn::punctuated::Punctuated<ClampedEnumVariant, syn::Token![,]>,
    },
}

impl Parse for ClampedEnumVariantField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Paren) {
            let content;
            let paren = syn::parenthesized!(content in input);
            let fork = content.fork();

            if fork.parse::<NumberArgRange>().is_ok() {
                let values = content.parse_terminated(NumberArgRange::parse, syn::Token![,])?;
                Ok(Self::Ranges { paren, values })
            } else {
                let values = content.parse_terminated(NumberArg::parse, syn::Token![,])?;
                Ok(Self::Values { paren, values })
            }
        } else if input.peek(syn::token::Bracket) {
            let content;
            let bracket = syn::bracketed!(content in input);
            let value_range: StrictNumberArgRange = content.parse()?;

            if value_range.0.start.is_none() || value_range.0.end.is_none() {
                return Err(input.error("Expected a range with both a start and an end"));
            }

            let content;
            let brace = syn::braced!(content in input);
            let variants = content.parse_terminated(ClampedEnumVariant::parse, syn::Token![,])?;

            Ok(Self::ClampedEnum {
                bracket: Some(bracket),
                value_range: Some(value_range),
                brace,
                variants,
            })
        } else if input.peek(syn::token::Brace) {
            let content;
            let brace = syn::braced!(content in input);
            let variants = content.parse_terminated(ClampedEnumVariant::parse, syn::Token![,])?;

            Ok(Self::ClampedEnum {
                bracket: None,
                value_range: None,
                brace,
                variants,
            })
        } else {
            Err(input.error("Expected a parenthesized list of values, a bracketed range, or a braced clamped enum"))
        }
    }
}

impl ToTokens for ClampedEnumVariantField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Values { paren, values } => {
                paren.surround(tokens, |tokens| {
                    values.to_tokens(tokens);
                });
            }
            Self::Ranges { paren, values } => {
                paren.surround(tokens, |tokens| {
                    values.to_tokens(tokens);
                });
            }
            Self::ClampedEnum {
                bracket,
                value_range,
                brace,
                variants,
            } => {
                if let Some(bracket) = bracket {
                    bracket.surround(tokens, |tokens| {
                        value_range.to_tokens(tokens);
                    });
                }

                brace.surround(tokens, |tokens| {
                    variants.to_tokens(tokens);
                });
            }
        }
    }
}

impl ClampedEnumVariantField {
    pub fn limits(
        &self,
        kind: NumberKind,
        lower_default: Option<NumberArg>,
        upper_default: Option<NumberArg>,
    ) -> (NumberArg, NumberArg) {
        match self {
            Self::Values { values, .. } => {
                let (mut lower_limit, mut upper_limit) =
                    (lower_default.clone(), upper_default.clone());

                for value in values.iter() {
                    lower_limit = lower_limit
                        .map(|lower_limit| lower_limit.min(value, kind))
                        .or(Some(value.clone()));

                    upper_limit = upper_limit
                        .map(|upper_limit| upper_limit.max(value, kind))
                        .or(Some(value.clone()));
                }

                (lower_limit.unwrap(), upper_limit.unwrap())
            }
            Self::Ranges { values, .. } => {
                let (mut lower_limit, mut upper_limit) =
                    (lower_default.clone(), upper_default.clone());

                for range in values.iter() {
                    let (start, end) = range.start_and_end_args(kind);

                    lower_limit = lower_limit.map_or_else(
                        || Some(start.clone()),
                        |lower_limit| Some(lower_limit.min(&start, kind)),
                    );

                    upper_limit = upper_limit.map_or_else(
                        || Some(end.clone()),
                        |upper_limit| Some(upper_limit.max(&end, kind)),
                    );
                }

                (lower_limit.unwrap(), upper_limit.unwrap())
            }
            Self::ClampedEnum {
                value_range,
                variants,
                ..
            } => {
                if let Some(range) = value_range {
                    range.start_and_end_args(kind)
                } else {
                    let (mut lower_limit, mut upper_limit) =
                        (lower_default.clone(), upper_default.clone());

                    for variant in variants.iter() {
                        let (start, end) = variant.field.limits(
                            kind,
                            lower_default.clone(),
                            upper_default.clone(),
                        );

                        lower_limit = lower_limit
                            .map(|lower_limit| lower_limit.min(&start, kind))
                            .or(Some(start));
                        upper_limit = upper_limit
                            .map(|upper_limit| upper_limit.max(&end, kind))
                            .or(Some(end));
                    }

                    (lower_limit.unwrap(), upper_limit.unwrap())
                }
            }
        }
    }
}

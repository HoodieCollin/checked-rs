use proc_macro2::{Span, TokenStream};
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

impl std::fmt::Debug for ClampedEnumVariantField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Values { values, .. } => {
                let values = values.iter().collect::<Vec<_>>();

                f.debug_tuple("Values").field(&values).finish()
            }
            Self::Ranges { values, .. } => {
                let values = values.iter().collect::<Vec<_>>();

                f.debug_tuple("Ranges").field(&values).finish()
            }
            Self::ClampedEnum {
                value_range,
                variants,
                ..
            } => {
                let variants = variants.iter().collect::<Vec<_>>();

                f.debug_struct("ClampedEnum")
                    .field("value_range", value_range)
                    .field("variants", &variants)
                    .finish()
            }
        }
    }
}

impl ClampedEnumVariantField {
    #[must_use]
    pub fn limits(
        &self,
        kind: NumberKind,
        hard_lower_limit: Option<NumberArg>,
        hard_upper_limit: Option<NumberArg>,
    ) -> syn::Result<NumberArgRange> {
        let (mut lower_limit, mut upper_limit) = NumberArg::LIMITS_INIT.clone();

        match self {
            Self::Values { values, .. } => {
                for value in values.iter() {
                    lower_limit = lower_limit.map_or_else(
                        || Some(value.clone()),
                        |lower_limit| Some(lower_limit.min(value, kind)),
                    );

                    upper_limit = upper_limit.map_or_else(
                        || Some(value.clone()),
                        |upper_limit| Some(upper_limit.max(value, kind)),
                    );
                }
            }
            Self::Ranges { values, .. } => {
                for range in values.iter() {
                    let start = range.start_arg(kind);
                    let end = range.end_arg(kind);

                    if lower_limit.is_none() && upper_limit.is_none() && range.is_full_range() {
                        lower_limit = hard_lower_limit
                            .as_ref()
                            .cloned()
                            .or_else(|| Some(NumberArg::new_min_constant(kind)));

                        upper_limit = hard_upper_limit
                            .as_ref()
                            .cloned()
                            .or_else(|| Some(NumberArg::new_max_constant(kind)));
                    } else {
                        lower_limit = lower_limit.map_or_else(
                            || Some(start.clone()),
                            |lower_limit| Some(lower_limit.min(&start, kind)),
                        );

                        upper_limit = upper_limit.map_or_else(
                            || Some(end.clone()),
                            |upper_limit| Some(upper_limit.max(&end, kind)),
                        );
                    }
                }
            }
            Self::ClampedEnum {
                value_range,
                variants,
                ..
            } => {
                for variant in variants.iter() {
                    let variant_limits = variant.field.limits(
                        kind,
                        value_range.as_ref().map(|range| range.start_arg(kind)),
                        value_range.as_ref().map(|range| range.end_arg(kind)),
                    )?;

                    let start = variant_limits.start_arg(kind);
                    let end = variant_limits.end_arg(kind);

                    lower_limit = lower_limit.map_or_else(
                        || Some(start.clone()),
                        |lower_limit| Some(lower_limit.min(&start, kind)),
                    );

                    upper_limit = upper_limit.map_or_else(
                        || Some(end.clone()),
                        |upper_limit| Some(upper_limit.max(&end, kind)),
                    );
                }
            }
        }

        if lower_limit.is_none() || upper_limit.is_none() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Field::Limits: No values in enum variant field",
            ));
        }

        let lower_limit = lower_limit.unwrap();
        let upper_limit = upper_limit.unwrap();

        if let Some(hard_lower_limit) = hard_lower_limit.map(|arg| arg.into_value(kind)) {
            let lower_limit = lower_limit.into_value(kind);

            if lower_limit < hard_lower_limit {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Enum variant field lower limit is below hard limit",
                ));
            }
        }

        if let Some(hard_upper_limit) = hard_upper_limit.map(|arg| arg.into_value(kind)) {
            let upper_limit = upper_limit.into_value(kind);

            if upper_limit > hard_upper_limit {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Enum variant field upper limit is above hard limit",
                ));
            }
        }

        Ok(NumberArgRange::new_inclusive(lower_limit, upper_limit))
    }
}

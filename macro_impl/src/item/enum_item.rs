use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{parse::Parse, parse_quote};

use crate::params::{
    kw, BehaviorArg, DerivedTraits, NumberArg, NumberKind, NumberRangeArg, NumberValue,
    NumberValueRange, NumberValueRangeSet, Params, SemiOrComma, StrictNumberRangeArg,
};

pub struct ClampedEnumItem {
    pub pound: syn::Token![#],
    pub bracket: syn::token::Bracket,
    pub integer: NumberKind,
    pub integer_semi: Option<SemiOrComma>,
    pub derived_traits: Option<DerivedTraits>,
    pub derived_semi: Option<SemiOrComma>,
    pub default_kw: Option<kw::default>,
    pub default_eq: Option<syn::Token![=]>,
    pub default_val: Option<NumberArg>,
    pub default_semi: Option<SemiOrComma>,
    pub behavior_kw: kw::behavior,
    pub behavior_eq: syn::Token![=],
    pub behavior_val: BehaviorArg,
    pub behavior_semi: Option<SemiOrComma>,
    pub vis: Option<syn::Visibility>,
    pub enum_token: syn::Token![enum],
    pub ident: syn::Ident,
    pub range_bracket: Option<syn::token::Bracket>,
    pub value_range: Option<NumberRangeArg>,
    pub brace: syn::token::Brace,
    pub variants: syn::punctuated::Punctuated<ClampedEnumVariant, syn::Token![,]>,
}

impl Parse for ClampedEnumItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pound = input.parse()?;

        let content;
        let bracket = syn::bracketed!(content in input);
        let integer = content.parse()?;
        let mut integer_semi = None;
        let mut derived_traits = None;
        let mut derived_semi = None;
        let mut default_kw = None;
        let mut default_eq = None;
        let mut default_val = None;
        let mut default_semi = None;
        let mut behavior_kw = None;
        let mut behavior_eq = None;
        let mut behavior_val = None;
        let mut behavior_semi = None;
        let mut vis = None;

        if !content.is_empty() {
            integer_semi = Some(content.parse()?);

            while !content.is_empty() {
                if content.peek(kw::derive) {
                    derived_traits = Some(content.parse()?);
                    derived_semi = if content.peek(syn::Token![;]) {
                        Some(content.parse()?)
                    } else {
                        None
                    };
                }

                if content.peek(kw::default) {
                    default_kw = Some(content.parse()?);
                    default_eq = Some(content.parse()?);
                    default_val = Some(content.parse()?);
                    default_semi = if content.peek(syn::Token![;]) {
                        Some(content.parse()?)
                    } else {
                        None
                    };
                }

                if content.peek(kw::behavior) {
                    behavior_kw = Some(content.parse()?);
                    behavior_eq = Some(content.parse()?);
                    behavior_val = Some(content.parse()?);
                    behavior_semi = if content.peek(syn::Token![;]) {
                        Some(content.parse()?)
                    } else {
                        None
                    };
                }
            }
        }

        if input.peek(syn::Token![pub]) {
            vis = Some(input.parse()?);
        }

        let enum_token = input.parse()?;
        let ident = input.parse()?;
        let mut range_bracket = None;
        let mut value_range = None;

        if input.peek(syn::token::Bracket) {
            let content;
            range_bracket = Some(syn::bracketed!(content in input));
            value_range = Some(content.parse()?);
        }

        let content;
        let brace = syn::braced!(content in input);
        let variants = content.parse_terminated(ClampedEnumVariant::parse, syn::Token![,])?;

        Ok(Self {
            pound,
            bracket,
            integer,
            integer_semi,
            derived_traits,
            derived_semi,
            default_kw,
            default_eq,
            default_val,
            default_semi,
            behavior_kw: behavior_kw.unwrap_or_else(|| parse_quote!(behavior)),
            behavior_eq: behavior_eq.unwrap_or_else(|| parse_quote!(=)),
            behavior_val: behavior_val.unwrap_or_else(|| parse_quote!(Panic)),
            behavior_semi,
            vis,
            enum_token,
            ident,
            range_bracket,
            value_range,
            brace,
            variants,
        })
    }
}

impl ClampedEnumItem {
    pub fn has_enum_token(input: syn::parse::ParseBuffer) -> syn::Result<bool> {
        let _ = input.parse::<syn::Token![#]>();
        let _content;
        syn::bracketed!(_content in input);

        Ok(input.peek(syn::Token![enum]))
    }

    // returns true if the coverage is complete
    pub fn check_coverage<'a, 'b: 'a>(
        out_exacts: Option<&'a mut HashSet<NumberValue>>,
        out_ranges: Option<&'a mut NumberValueRangeSet>,
        parent_lower_limit: Option<NumberValue>,
        parent_upper_limit: Option<NumberValue>,
        kind: NumberKind,
        variants: impl Iterator<Item = &'b ClampedEnumVariant>,
    ) -> syn::Result<bool> {
        let mut has_full_range = false;
        let mut exacts = HashSet::with_capacity(64);
        let mut ranges = NumberValueRangeSet::new_with_step_fns();

        for variant in variants {
            match &variant.field {
                ClampedEnumVariantField::Values { values, .. } => {
                    for val in values.iter() {
                        let val = val.into_value(kind);

                        if let Some(lower_limit) = parent_lower_limit {
                            if val < lower_limit {
                                return Err(syn::Error::new(
                                    Span::call_site(),
                                    format!("Value below lower limit in clamped enum {}", val),
                                ));
                            }
                        }

                        if let Some(upper_limit) = parent_upper_limit {
                            if val > upper_limit {
                                return Err(syn::Error::new(
                                    Span::call_site(),
                                    format!("Value above upper limit in clamped enum {}", val),
                                ));
                            }
                        }

                        if !exacts.insert(val) {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                format!("Duplicate value in clamped enum {}", val),
                            ));
                        }
                    }
                }
                ClampedEnumVariantField::Ranges { values, .. } => {
                    if values.len() == 1 {
                        let range = values.first().unwrap();

                        if range.is_full_range() {
                            if has_full_range {
                                return Err(syn::Error::new(
                                    Span::call_site(),
                                    "Multiple full ranges in clamped enum",
                                ));
                            } else {
                                has_full_range = true;
                                continue;
                            }
                        }

                        // intentionally fall through to looping over the ranges
                    }

                    for range in values.iter() {
                        if range.is_full_range() {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                "Full ranges cannot be mixed with other ranges in the same variant of within a clamped enum",
                            ));
                        }

                        let std_range = range
                            .to_value_range(kind, parent_lower_limit, parent_upper_limit)?
                            .to_std_inclusive_range(None, None)?;

                        if ranges.overlaps(&std_range) {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                format!("Overlapping range in clamped enum {}", range),
                            ));
                        } else {
                            ranges.insert(std_range);
                        }
                    }
                }
                ClampedEnumVariantField::ClampedEnum {
                    value_range,
                    variants,
                    ..
                } => {
                    let mut lower_limit = None;
                    let mut upper_limit = None;
                    let mut inner_exacts = HashSet::with_capacity(64);
                    let mut inner_ranges = NumberValueRangeSet::new_with_step_fns();

                    if let Some(range) = value_range {
                        let std_range = range
                            .to_value_range(kind, parent_lower_limit, parent_upper_limit)?
                            .to_std_inclusive_range(None, None)?;

                        if ranges.overlaps(&std_range) {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                format!("Overlapping range in clamped enum {}", range),
                            ));
                        } else {
                            ranges.insert(std_range);
                            lower_limit = range
                                .0
                                .start
                                .as_ref()
                                .map(|val| val.into_value(kind))
                                .or(parent_lower_limit);
                            upper_limit = range
                                .0
                                .end
                                .as_ref()
                                .map(|val| val.into_value(kind))
                                .or(parent_upper_limit);
                        }
                    }

                    let full_coverage = Self::check_coverage(
                        Some(&mut inner_exacts),
                        Some(&mut inner_ranges),
                        lower_limit,
                        upper_limit,
                        kind,
                        variants.iter(),
                    )?;

                    if let Some(val) = exacts.intersection(&inner_exacts).next() {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            format!("Duplicate value in clamped enum {}", val),
                        ));
                    } else {
                        exacts.extend(inner_exacts);
                    }

                    if full_coverage {
                        let full_inner_range =
                            NumberValueRange::from_values(lower_limit, upper_limit, kind)?
                                .to_std_inclusive_range(None, None)?;

                        if ranges.overlaps(&full_inner_range) {
                            return Err(syn::Error::new(
                                Span::call_site(),
                                format!("Overlapping range in clamped enum {:?}", full_inner_range),
                            ));
                        } else {
                            ranges.insert(full_inner_range);
                        }
                    } else if let Some(range) = ranges.intersection(&inner_ranges).next() {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            format!("Overlapping range in clamped enum {:?}", range),
                        ));
                    } else {
                        for range in inner_ranges.iter() {
                            ranges.insert(range.clone());
                        }
                    }
                }
            }
        }

        if let Some(out_exacts) = out_exacts {
            if let Some(val) = out_exacts.intersection(&exacts).next() {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Duplicate value in clamped enum {}", val),
                ));
            } else {
                out_exacts.extend(exacts);
            }
        }

        let full_start = parent_lower_limit
            .unwrap_or_else(|| NumberArg::new_min_constant(kind).into_value(kind));

        let full_end = parent_upper_limit
            .unwrap_or_else(|| NumberArg::new_max_constant(kind).into_value(kind));

        let full_range = NumberValueRange::from_values(Some(full_start), Some(full_end), kind)?
            .to_std_inclusive_range(None, None)?;

        if has_full_range {
            if let Some(out_ranges) = out_ranges {
                if out_ranges.overlaps(&full_range) {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!("Overlapping range in clamped enum {:?}", full_range),
                    ));
                } else {
                    out_ranges.insert(full_range);
                }
            }

            return Ok(true);
        } else {
            if let Some(out_ranges) = out_ranges {
                if let Some(range) = out_ranges.intersection(&ranges).next() {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!("Overlapping range in clamped enum {:?}", range),
                    ));
                }

                for range in ranges.iter() {
                    out_ranges.insert(range.clone());
                }
            }

            return Ok(ranges.gaps(&full_range).next().is_none());
        }
    }

    pub fn limits(&self) -> (Option<NumberArg>, Option<NumberArg>) {
        let mut parent_lower_limit = None;
        let mut parent_upper_limit = None;

        if let Some(range) = &self.value_range {
            parent_lower_limit = range.start.clone();
            parent_upper_limit = range.end.clone();
        }

        self.variants.iter().fold(
            (parent_lower_limit.clone(), parent_upper_limit.clone()),
            |acc, variant| {
                let (start, end) = variant.field.limits(
                    self.integer,
                    parent_lower_limit.clone(),
                    parent_upper_limit.clone(),
                );

                (
                    acc.0
                        .map(|lower_limit| lower_limit.min(&start, self.integer))
                        .or(Some(start)),
                    acc.1
                        .map(|upper_limit| upper_limit.max(&end, self.integer))
                        .or(Some(end)),
                )
            },
        )
    }

    pub fn params(&self) -> syn::Result<Params> {
        let (total_lower_limit, total_upper_limit) = self.limits();

        Ok(Params {
            integer: self.integer,
            derived_traits: self.derived_traits.clone(),
            vis: self.vis.clone().unwrap_or(syn::Visibility::Inherited),
            ident: self.ident.clone(),
            as_soft_or_hard: None,
            default_val: self.default_val.clone(),
            behavior_val: self.behavior_val.clone(),
            lower_limit: total_lower_limit.clone(),
            upper_limit: total_upper_limit.clone(),
            full_coverage: Self::check_coverage(
                None,
                None,
                total_lower_limit.map(|arg| arg.into_value(self.integer)),
                total_upper_limit.map(|arg| arg.into_value(self.integer)),
                self.integer,
                self.variants.iter(),
            )?,
        })
    }
}

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

#[derive(Clone)]
pub enum ClampedEnumVariantField {
    Values {
        paren: syn::token::Paren,
        values: syn::punctuated::Punctuated<NumberArg, syn::Token![,]>,
    },
    Ranges {
        paren: syn::token::Paren,
        values: syn::punctuated::Punctuated<NumberRangeArg, syn::Token![,]>,
    },
    ClampedEnum {
        bracket: Option<syn::token::Bracket>,
        value_range: Option<StrictNumberRangeArg>,
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

            if fork.parse::<NumberRangeArg>().is_ok() {
                let values = content.parse_terminated(NumberRangeArg::parse, syn::Token![,])?;
                Ok(Self::Ranges { paren, values })
            } else {
                let values = content.parse_terminated(NumberArg::parse, syn::Token![,])?;
                Ok(Self::Values { paren, values })
            }
        } else if input.peek(syn::token::Bracket) {
            let content;
            let bracket = syn::bracketed!(content in input);
            let value_range: StrictNumberRangeArg = content.parse()?;

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

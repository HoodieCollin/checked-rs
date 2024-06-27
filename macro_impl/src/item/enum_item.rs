use std::collections::HashSet;

use proc_macro2::Span;
use syn::{parse::Parse, parse_quote};

use crate::{
    params::{
        kw, BehaviorArg, DerivedTraits, NumberArg, NumberArgRange, NumberKind, NumberValue,
        NumberValueRange, Params, SemiOrComma,
    },
    range_seq::RangeSeq,
};

pub mod field;
pub mod variant;

pub use field::*;
pub use variant::*;

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
    pub behavior: BehaviorArg,
    pub behavior_semi: Option<SemiOrComma>,
    pub vis: Option<syn::Visibility>,
    pub enum_token: syn::Token![enum],
    pub ident: syn::Ident,
    pub range_bracket: Option<syn::token::Bracket>,
    pub value_range: Option<NumberArgRange>,
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
        let mut behavior = None;
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
                    behavior = Some(content.parse()?);
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
            behavior: behavior.unwrap_or_else(|| parse_quote!(Panic)),
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
        parent_exacts: Option<&'a mut HashSet<NumberValue>>,
        parent_range_seq: Option<&'a mut RangeSeq>,
        parent_lower_limit: Option<NumberValue>,
        parent_upper_limit: Option<NumberValue>,
        kind: NumberKind,
        variants: impl Iterator<Item = &'b ClampedEnumVariant>,
    ) -> syn::Result<bool> {
        let mut exacts = HashSet::new();
        let mut outer_range_seq = RangeSeq::new(kind);

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
                    for range in values.iter() {
                        outer_range_seq.insert(range.to_value_range(kind)?)?;
                    }
                }
                ClampedEnumVariantField::ClampedEnum {
                    value_range,
                    variants,
                    ..
                } => {
                    let mut lower_limit = None;
                    let mut upper_limit = None;
                    let mut inner_exacts = HashSet::new();
                    let mut inner_range_seq = RangeSeq::new(kind);

                    if let Some(range) = value_range {
                        lower_limit = Some(range.first_val(kind));
                        upper_limit = Some(range.last_val(kind));
                    }

                    let full_coverage = Self::check_coverage(
                        Some(&mut inner_exacts),
                        Some(&mut inner_range_seq),
                        lower_limit,
                        upper_limit,
                        kind,
                        variants.iter(),
                    )?;

                    if let Some(val) = exacts.intersection(&inner_exacts).next() {
                        return Err(syn::Error::new(
                            Span::call_site(),
                            format!("Nested[1]: Duplicate value in clamped enum {}", val),
                        ));
                    } else {
                        exacts.extend(inner_exacts);
                    }

                    if full_coverage {
                        outer_range_seq.insert(NumberValueRange::new_inclusive(
                            lower_limit,
                            upper_limit,
                            kind,
                        )?)?;
                    } else {
                        for range in inner_range_seq.ranges() {
                            outer_range_seq.insert(range)?;
                        }
                    }
                }
            }
        }

        if let Some(parent_exacts) = parent_exacts {
            if let Some(val) = parent_exacts.intersection(&exacts).next() {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("Outer: Duplicate value in clamped enum {}", val),
                ));
            } else {
                parent_exacts.extend(exacts);
            }
        }

        let full_start = parent_lower_limit
            .unwrap_or_else(|| NumberArg::new_min_constant(kind).into_value(kind));

        let full_end = parent_upper_limit
            .unwrap_or_else(|| NumberArg::new_max_constant(kind).into_value(kind));

        if outer_range_seq.has_full_range() {
            if let Some(parent_range_seq) = parent_range_seq {
                let full_range =
                    NumberValueRange::new_inclusive(Some(full_start), Some(full_end), kind)?;

                parent_range_seq.insert(full_range)?;
            }

            return Ok(true);
        } else if let Some(parent_range_seq) = parent_range_seq {
            for range in outer_range_seq.ranges() {
                parent_range_seq.insert(range)?;
            }
        }

        return Ok(outer_range_seq.has_gaps());
    }

    pub fn limits(&self) -> syn::Result<NumberArgRange> {
        let kind = self.integer;
        let hard_lower_limit = self.value_range.as_ref().map(|range| range.start_arg(kind));
        let hard_upper_limit = self.value_range.as_ref().map(|range| range.end_arg(kind));

        let (mut lower_limit, mut upper_limit) = NumberArg::LIMITS_INIT.clone();

        for variant in self.variants.iter() {
            let variant_limits =
                variant
                    .field
                    .limits(kind, hard_lower_limit.clone(), hard_upper_limit.clone())?;

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

        if lower_limit.is_none() || upper_limit.is_none() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Item::Limits: No values in enum variant field",
            ));
        }

        let lower_limit = lower_limit.unwrap();
        let upper_limit = upper_limit.unwrap();

        if let Some(hard_lower_limit) = hard_lower_limit.map(|arg| arg.into_value(kind)) {
            if lower_limit.into_value(kind) < hard_lower_limit {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Enum variant lower limit is below hard limit",
                ));
            }
        }

        if let Some(hard_upper_limit) = hard_upper_limit.map(|arg| arg.into_value(kind)) {
            if upper_limit.into_value(kind) > hard_upper_limit {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Enum variant upper limit is above hard limit",
                ));
            }
        }

        Ok(NumberArgRange::new_inclusive(lower_limit, upper_limit))
    }

    pub fn params(&self) -> syn::Result<Params> {
        let kind = self.integer;
        let limits = self.limits()?;

        let total_lower_limit = limits.first_val(kind);
        let total_upper_limit = limits.last_val(kind);

        let mut parent_exacts = HashSet::new();
        let mut parent_range_seq = RangeSeq::new(kind);

        let this = Params {
            integer: kind,
            derived_traits: self.derived_traits.clone(),
            vis: self.vis.clone().unwrap_or(syn::Visibility::Inherited),
            ident: self.ident.clone(),
            as_soft_or_hard: None,
            default_val: self.default_val.as_ref().map(|arg| arg.into_value(kind)),
            behavior: self.behavior.clone(),
            lower_limit_val: total_lower_limit,
            upper_limit_val: total_upper_limit,
            full_coverage: Self::check_coverage(
                Some(&mut parent_exacts),
                Some(&mut parent_range_seq),
                Some(total_lower_limit),
                Some(total_upper_limit),
                kind,
                self.variants.iter(),
            )?,
        };

        Ok(this)
    }
}

use syn::{parse::Parse, parse_quote};

use crate::{
    params::{
        kw, AsSoftOrHard, BehaviorArg, DerivedTraits, NumberArg, NumberKind, Params, SemiOrComma,
    },
    range_seq::RangeSeq,
};

pub mod field;

pub use field::*;

pub struct ClampedStructItem {
    pub pound: syn::Token![#],
    pub bracket: syn::token::Bracket,
    pub integer: NumberKind,
    pub as_soft_or_hard: Option<AsSoftOrHard>,
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
    pub struct_token: syn::Token![struct],
    pub ident: syn::Ident,
    pub field: ClampedStructField,
    pub final_semi: Option<syn::Token![;]>,
}

impl Parse for ClampedStructItem {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let pound = input.parse()?;

        let content;
        let bracket = syn::bracketed!(content in input);
        let integer = content.parse()?;
        let mut as_soft_or_hard = None;
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
            if content.peek(syn::Token![as]) {
                as_soft_or_hard = Some(content.parse()?);
            }

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
        }

        if input.peek(syn::Token![pub]) {
            vis = Some(input.parse()?);
        }

        Ok(Self {
            pound,
            bracket,
            integer,
            as_soft_or_hard,
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
            struct_token: input.parse()?,
            ident: input.parse()?,
            field: input.parse()?,
            final_semi: if input.is_empty() {
                None
            } else {
                Some(input.parse()?)
            },
        })
    }
}

impl ClampedStructItem {
    pub fn params(&self) -> syn::Result<Params> {
        let kind = self.integer;
        let (mut lower_limit, mut upper_limit) = NumberArg::LIMITS_INIT.clone();

        let mut range_seq = RangeSeq::new(kind);

        for range in self.field.ranges.iter() {
            range_seq.insert(range.to_value_range(kind)?)?;

            let start = range.start_arg(kind);
            let end = range.end_arg(kind);

            lower_limit = lower_limit.map_or_else(
                || Some(start.clone()),
                |lower_limit| Some(lower_limit.min(&start, kind)),
            );

            upper_limit = upper_limit.map_or_else(
                || Some(end.clone()),
                |upper_limit| Some(upper_limit.max(&end, kind)),
            );
        }

        Ok(Params {
            integer: self.integer,
            derived_traits: self.derived_traits.clone(),
            vis: self.vis.clone().unwrap_or(syn::Visibility::Inherited),
            ident: self.ident.clone(),
            as_soft_or_hard: self.as_soft_or_hard.clone(),
            default_val: self.default_val.as_ref().map(|arg| arg.into_value(kind)),
            behavior: self.behavior_val.clone(),
            lower_limit_val: lower_limit
                .or_else(|| Some(NumberArg::new_min_constant(kind)))
                .map(|arg| arg.into_value(kind))
                .unwrap(),
            upper_limit_val: upper_limit
                .or_else(|| Some(NumberArg::new_max_constant(kind)))
                .map(|arg| arg.into_value(kind))
                .unwrap(),
            full_coverage: !range_seq.has_gaps(),
        })
    }
}

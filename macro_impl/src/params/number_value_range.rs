use std::ops::{RangeFrom, RangeInclusive, RangeToInclusive};

use proc_macro2::Span;
use rangemap::{RangeInclusiveSet, StepFns};

use super::{NumberArg, NumberKind, NumberValue};

pub type NumberValueRangeSet = RangeInclusiveSet<NumberValue, NumberValueStepFns>;

pub struct NumberValueStepFns;

impl StepFns<NumberValue> for NumberValueStepFns {
    fn add_one(start: &NumberValue) -> NumberValue {
        start.add_usize(1)
    }

    fn sub_one(start: &NumberValue) -> NumberValue {
        start.sub_usize(1)
    }
}

#[derive(Clone)]
pub enum NumberValueRange {
    Full(NumberKind),
    From(RangeFrom<NumberValue>),
    To(RangeToInclusive<NumberValue>),
    Inclusive(RangeInclusive<NumberValue>),
}

impl NumberValueRange {
    fn check_matching_kinds(
        a: impl Into<NumberKind> + std::fmt::Debug + Clone,
        b: impl Into<NumberKind> + std::fmt::Debug + Clone,
    ) -> syn::Result<()> {
        let a_kind: NumberKind = a.clone().into();
        let b_kind: NumberKind = b.clone().into();

        if a_kind != b_kind {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("Number kinds do not match: {:?} != {:?}", a, b),
            ));
        }

        Ok(())
    }

    pub fn from_values(
        start: Option<NumberValue>,
        end: Option<NumberValue>,
        kind: NumberKind,
    ) -> syn::Result<Self> {
        Ok(match (start, end) {
            (Some(start), Some(end)) => {
                Self::check_matching_kinds(kind, &start)?;
                Self::check_matching_kinds(kind, &end)?;
                Self::Inclusive(start..=end)
            }
            (Some(start), None) => {
                Self::check_matching_kinds(kind, &start)?;
                Self::From(start..)
            }
            (None, Some(end)) => {
                Self::check_matching_kinds(kind, &end)?;
                Self::To(..=end)
            }
            (None, None) => Self::Full(kind),
        })
    }

    pub fn to_std_inclusive_range(
        &self,
        start_default: Option<NumberValue>,
        end_default: Option<NumberValue>,
    ) -> syn::Result<RangeInclusive<NumberValue>> {
        match self {
            Self::Full(kind) => {
                let start = start_default
                    .unwrap_or_else(|| NumberArg::new_min_constant(*kind).into_value(*kind));

                Self::check_matching_kinds(&start, *kind)?;

                let end = end_default
                    .unwrap_or_else(|| NumberArg::new_max_constant(*kind).into_value(*kind));

                Self::check_matching_kinds(&end, *kind)?;

                Ok(start..=end)
            }
            Self::From(range) => {
                let start = range.start.clone();
                let kind = start.kind();

                let end = end_default
                    .unwrap_or_else(|| NumberArg::new_max_constant(kind).into_value(kind));

                Self::check_matching_kinds(&end, kind)?;

                Ok(start..=end)
            }
            Self::To(range) => {
                let end = range.end.clone();
                let kind = end.kind();

                let start = start_default
                    .unwrap_or_else(|| NumberArg::new_min_constant(kind).into_value(kind));

                Self::check_matching_kinds(&start, kind)?;

                Ok(start..=end)
            }
            Self::Inclusive(range) => {
                let start = range.start();
                let end = range.end();

                Self::check_matching_kinds(start, end)?;

                Ok(*start..=*end)
            }
        }
    }
}

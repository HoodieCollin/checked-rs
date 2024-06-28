use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

use super::{NumberArg, NumberArgRange, NumberKind, NumberValue};

#[derive(Debug, Clone)]
pub enum NumberValueRange {
    Full(NumberKind),
    From(RangeFrom<NumberValue>),
    ToExclusive(RangeTo<NumberValue>),
    ToInclusive(RangeToInclusive<NumberValue>),
    Exclusive(Range<NumberValue>),
    Inclusive(RangeInclusive<NumberValue>),
}

impl From<NumberKind> for NumberValueRange {
    fn from(kind: NumberKind) -> Self {
        Self::Full(kind)
    }
}

impl From<&NumberKind> for NumberValueRange {
    fn from(kind: &NumberKind) -> Self {
        Self::Full(*kind)
    }
}

impl From<RangeFrom<NumberValue>> for NumberValueRange {
    fn from(range: RangeFrom<NumberValue>) -> Self {
        Self::From(range)
    }
}

impl From<&RangeFrom<NumberValue>> for NumberValueRange {
    fn from(range: &RangeFrom<NumberValue>) -> Self {
        Self::From(range.clone())
    }
}

impl From<RangeTo<NumberValue>> for NumberValueRange {
    fn from(range: RangeTo<NumberValue>) -> Self {
        Self::ToExclusive(range)
    }
}

impl From<&RangeTo<NumberValue>> for NumberValueRange {
    fn from(range: &RangeTo<NumberValue>) -> Self {
        Self::ToExclusive(range.clone())
    }
}

impl From<RangeToInclusive<NumberValue>> for NumberValueRange {
    fn from(range: RangeToInclusive<NumberValue>) -> Self {
        Self::ToInclusive(range)
    }
}

impl From<&RangeToInclusive<NumberValue>> for NumberValueRange {
    fn from(range: &RangeToInclusive<NumberValue>) -> Self {
        Self::ToInclusive(range.clone())
    }
}

impl From<Range<NumberValue>> for NumberValueRange {
    fn from(range: Range<NumberValue>) -> Self {
        Self::Exclusive(range)
    }
}

impl From<&Range<NumberValue>> for NumberValueRange {
    fn from(range: &Range<NumberValue>) -> Self {
        Self::Exclusive(range.clone())
    }
}

impl From<RangeInclusive<NumberValue>> for NumberValueRange {
    fn from(range: RangeInclusive<NumberValue>) -> Self {
        Self::Inclusive(range)
    }
}

impl From<&RangeInclusive<NumberValue>> for NumberValueRange {
    fn from(range: &RangeInclusive<NumberValue>) -> Self {
        Self::Inclusive(range.clone())
    }
}

impl ToTokens for NumberValueRange {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min = self.first_val();
        let max = self.last_val();

        tokens.extend(quote! {
            #min..=#max
        });
    }
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

    pub fn kind(&self) -> NumberKind {
        match self {
            Self::Full(kind) => *kind,
            Self::From(range) => range.start.kind(),
            Self::ToExclusive(range) => range.end.kind(),
            Self::ToInclusive(range) => range.end.kind(),
            Self::Exclusive(range) => range.start.kind(),
            Self::Inclusive(range) => range.start().kind(),
        }
    }

    pub fn first_val(&self) -> NumberValue {
        match self {
            Self::From(range) => range.start,
            Self::Inclusive(range) => *range.start(),
            Self::Exclusive(range) => range.start,
            _ => {
                let kind = self.kind();
                NumberArg::new_min_constant(kind).into_value(kind)
            }
        }
    }

    pub fn last_val(&self) -> NumberValue {
        match self {
            Self::ToExclusive(range) => range.end.sub_usize(1),
            Self::ToInclusive(range) => range.end,
            Self::Exclusive(range) => range.end.sub_usize(1),
            Self::Inclusive(range) => *range.end(),
            _ => {
                let kind = self.kind();
                NumberArg::new_max_constant(kind).into_value(kind)
            }
        }
    }

    pub fn contains(&self, val: &NumberValue) -> bool {
        if *val >= self.first_val() && *val <= self.first_val() {
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn new_inclusive(
        start: Option<NumberValue>,
        end: Option<NumberValue>,
        kind: NumberKind,
    ) -> syn::Result<Self> {
        Ok(match (start, end) {
            (Some(start), Some(end)) => {
                Self::check_matching_kinds(start, kind)?;
                Self::check_matching_kinds(end, kind)?;
                Self::Inclusive(start..=end)
            }
            (Some(start), None) => {
                Self::check_matching_kinds(start, kind)?;
                Self::From(start..)
            }
            (None, Some(end)) => {
                Self::check_matching_kinds(end, kind)?;
                Self::ToInclusive(..=end)
            }
            (None, None) => Self::Full(kind),
        })
    }

    #[must_use]
    pub fn new_exclusive(
        start: Option<NumberValue>,
        end: Option<NumberValue>,
        kind: NumberKind,
    ) -> syn::Result<Self> {
        Ok(match (start, end) {
            (Some(start), Some(end)) => {
                Self::check_matching_kinds(start, kind)?;
                Self::check_matching_kinds(end, kind)?;
                Self::Exclusive(start..end)
            }
            (Some(start), None) => {
                Self::check_matching_kinds(start, kind)?;
                Self::From(start..)
            }
            (None, Some(end)) => {
                Self::check_matching_kinds(end, kind)?;
                Self::ToExclusive(..end)
            }
            (None, None) => Self::Full(kind),
        })
    }

    #[must_use]
    pub fn from_arg_range(arg_range: NumberArgRange, kind: NumberKind) -> syn::Result<Self> {
        let NumberArgRange {
            start,
            end,
            dot_dot_eq,
            ..
        } = arg_range;

        let inclusive = dot_dot_eq.is_some();
        let start = start.map(|arg| arg.into_value(kind));
        let end = end.map(|arg| arg.into_value(kind));

        Ok(match (start, end) {
            (None, None) => Self::Full(kind),
            (Some(start), None) => {
                Self::check_matching_kinds(kind, &start)?;
                Self::From(start..)
            }
            (Some(start), Some(end)) => {
                Self::check_matching_kinds(kind, &start)?;
                Self::check_matching_kinds(kind, &end)?;

                if inclusive {
                    Self::Inclusive(start..=end)
                } else {
                    Self::Exclusive(start..end)
                }
            }
            (None, Some(end)) => {
                Self::check_matching_kinds(kind, &end)?;

                if inclusive {
                    Self::ToInclusive(..=end)
                } else {
                    Self::ToExclusive(..end)
                }
            }
        })
    }
}

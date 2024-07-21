#![doc = include_str!("../README.md")]

use std::{
    num,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Rem, Sub},
};

use clamp::ValueRangeInclusive;
pub mod clamp;
pub mod guard;
pub mod view;

mod reexports {
    #[doc(hidden)]
    pub use anyhow::{anyhow, bail, ensure, format_err, Chain, Context, Error, Result};
    #[doc(hidden)]
    pub use serde;
}

pub mod prelude {
    pub use crate::reexports::*;

    pub use crate::{
        clamp::*, commit_or_bail, view::*, Behavior, InherentBehavior, InherentLimits,
        OpBehaviorParams,
    };

    pub use checked_rs_macros::clamped;
}

#[derive(Debug, Clone)]
pub enum OpBehaviorParams<T: 'static + Copy + Eq + Ord + InherentLimits<T>> {
    Simple {
        min: T,
        max: T,
    },
    ExactsOnly(&'static [T]),
    RangesOnly(&'static [ValueRangeInclusive<T>]),
    ExactsAndRanges {
        exacts: &'static [T],
        ranges: &'static [ValueRangeInclusive<T>],
    },
}

pub trait Behavior: Copy + 'static {
    fn add<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Add<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Add<Output = num::Saturating<T>>,
        <num::Saturating<T> as Add>::Output: Eq + Ord + Into<num::Saturating<T>>;
    fn sub<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Sub<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Sub<Output = num::Saturating<T>>,
        <num::Saturating<T> as Sub>::Output: Eq + Ord + Into<num::Saturating<T>>;
    fn mul<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Mul<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Mul<Output = num::Saturating<T>>,
        <num::Saturating<T> as Mul>::Output: Eq + Ord + Into<num::Saturating<T>>;
    fn div<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Div<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Div<Output = num::Saturating<T>>,
        <num::Saturating<T> as Div>::Output: Eq + Ord + Into<num::Saturating<T>>;
    fn rem<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Rem<Output = T> + Sub<Output = T>,
        <T as Rem>::Output: Eq + Ord + Into<T>,
        <T as Sub>::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Rem<Output = num::Saturating<T>>,
        <num::Saturating<T> as Rem>::Output: Eq + Ord + Into<num::Saturating<T>>;
    fn bitand<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: BitAnd<Output = T> + Sub<Output = T>,
        <T as BitAnd>::Output: Eq + Ord + Into<T>,
        <T as Sub>::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: BitAnd<Output = num::Saturating<T>>,
        <num::Saturating<T> as BitAnd>::Output: Eq + Ord + Into<num::Saturating<T>>;
    fn bitor<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: BitOr<Output = T> + Sub<Output = T>,
        <T as BitOr>::Output: Eq + Ord + Into<T>,
        <T as Sub>::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: BitOr<Output = num::Saturating<T>>,
        <num::Saturating<T> as BitOr>::Output: Eq + Ord + Into<num::Saturating<T>>;
    fn bitxor<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: BitXor<Output = T> + Sub<Output = T>,
        <T as BitXor>::Output: Eq + Ord + Into<T>,
        <T as Sub>::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: BitXor<Output = num::Saturating<T>>,
        <num::Saturating<T> as BitXor>::Output: Eq + Ord + Into<num::Saturating<T>>;
    fn neg<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        val: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Neg<Output = T> + Sub<Output = T>,
        <T as Neg>::Output: Eq + Ord + Into<T>,
        <T as Sub>::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Neg<Output = num::Saturating<T>>,
        <num::Saturating<T> as Neg>::Output: Eq + Ord + Into<num::Saturating<T>>;
    fn not<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        val: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Not<Output = T> + Sub<Output = T>,
        <T as Not>::Output: Eq + Ord + Into<T>,
        <T as Sub>::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Not<Output = num::Saturating<T>>,
        <num::Saturating<T> as Not>::Output: Eq + Ord + Into<num::Saturating<T>>;
}

pub trait InherentLimits<T>: 'static {
    const MIN: Self;
    const MAX: Self;
    const MIN_INT: T;
    const MAX_INT: T;

    fn is_zero(&self) -> bool;
    fn is_negative(&self) -> bool;
    fn is_positive(&self) -> bool;
}

pub trait InherentBehavior: 'static {
    type Behavior: Behavior;
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    clamped! {
        #[usize; derive(Debug)]
        enum DoubleSentinel {
            Zero(0),
            Valid(..),
            Invalid(usize::MAX),
        }
    }

    #[test]
    fn test_enum_simple() {
        let value = DoubleSentinel::new(0);

        assert!(value.is_some());

        let mut value = value.unwrap();

        value += 1;

        assert_eq!(value, 1);
        assert!(value.is_valid());

        value -= 1;
        assert!(value.is_zero());

        value += usize::MAX;
        assert!(value.is_invalid());
    }

    clamped! {
        #[isize; derive(Debug)]
        enum SignedNumbers {
            Min(isize::MIN),
            Neg(..0),
            Zero(0),
            Pos(0..),
            Max(isize::MAX),
        }
    }

    // #[test]
    // fn test_enum_non_comprehensive() {
    //     clamped! {
    //         #[usize]
    //         enum TenTwentyThirty {
    //             Ten(10),
    //             Twenty(20),
    //             Thirty(30),
    //         }
    //     }
    // }

    // #[test]
    // fn test_enum_multiple_exacts() {
    //     clamped! {
    //         #[usize]
    //         enum SpecificValues {
    //             OneTwoOrSeven(1, 2, 7),
    //             AnythingElse(..),
    //         }
    //     }
    // }

    // #[test]
    // fn test_enum_multiple_ranges() {
    //     clamped! {
    //         #[usize]
    //         enum HundredToThousand {
    //             Valid(..),
    //             Invalid(..100, 1000..)
    //         }
    //     }
    // }

    clamped! {
        #[usize]
        enum ResponseCode {
            Success[200..300] {
                Okay(200),
                Created(201),
                Accepted(202),
                Unknown(..),
            },
            Error {
                Client[400..500] {
                    BadRequest(400),
                    Unauthorized(401),
                    PaymentRequired(402),
                    Forbidden(403),
                    NotFound(404),
                    Unknown(..)
                },
                Server[500..600] {
                    Internal(500),
                    NotImplemented(501),
                    BadGateway(502),
                    ServiceUnavailable(503),
                    GatewayTimeout(504),
                    Unknown(..)
                }
            }
        }
    }

    #[test]
    fn test_enum_nested() {}

    // #[test]
    // fn test_struct_soft() {
    //     clamped! {
    //         #[usize as Soft]
    //         struct TenOrLess(..=10);
    //     }
    // }

    clamped! {
        #[usize as Hard; derive(Debug)]
        struct TenOrMore(10..);
    }

    #[test]
    fn test_struct_hard() {
        let value = TenOrMore::new(10);

        assert!(value.is_some());

        let mut value = value.unwrap();

        value += 1;

        assert_eq!(value, 11);
    }

    #[test]
    #[should_panic]
    fn test_struct_hard_overflow() {
        let value = TenOrMore::new(10);

        assert!(value.is_some());

        let mut value = value.unwrap();

        value -= 1;
    }

    clamped! {
        #[usize as Hard; derive(Debug)]
        struct LessThanTenOrBetween999And2000(..10, 1000..2000);
    }

    #[test]
    fn test_struct_multiple_ranges() {
        let value = LessThanTenOrBetween999And2000::new(5);

        assert!(value.is_some());

        let mut value = value.unwrap();

        value += 3;

        assert_eq!(value, 8);

        value += 1000;

        assert_eq!(value, 1008);
    }
}

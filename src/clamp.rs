use std::{
    num,
    ops::{
        Add, BitAnd, BitOr, BitXor, Div, Mul, RangeFrom, RangeInclusive, RangeToInclusive, Rem, Sub,
    },
};

use crate::{InherentBehavior, InherentLimits};
use anyhow::Result;

pub unsafe trait ExactValues<T: 'static + Copy + Eq + Ord>:
    'static + Default + Eq + Ord
{
    const VALUES: &'static [T];

    fn contains_value(val: T) -> bool {
        Self::VALUES.contains(&val)
    }
}

pub unsafe trait ClampedInteger<T: Copy>:
    'static + Default + Eq + Ord + InherentLimits<T>
{
    fn from_primitive(val: T) -> Result<Self>;
    fn as_primitive(&self) -> &T;

    fn into_primitive(&self) -> T {
        *self.as_primitive()
    }
}

macro_rules! impl_clamped_integer_for_basic_types {
    ($($ty:ty),* $(,)?) => {
        $(
            impl InherentLimits<$ty> for $ty {
                const MIN: $ty = <$ty>::MIN;
                const MAX: $ty = <$ty>::MAX;
                const MIN_INT: $ty = <$ty>::MIN;
                const MAX_INT: $ty = <$ty>::MAX;
            }

            unsafe impl ClampedInteger<$ty> for $ty {
                fn from_primitive(val: $ty) -> Result<Self> {
                    Ok(val)
                }

                fn as_primitive(&self) -> &$ty {
                    self
                }
            }
        )*
    };
}

impl_clamped_integer_for_basic_types! {
    i8, i16, i32, i64, i128,
    u8, u16, u32, u64, u128,
    isize, usize,
}

#[derive(Clone)]
pub enum ValueRange<T: 'static + Copy + Eq + Ord + InherentLimits<T>> {
    Full(std::marker::PhantomData<T>),
    From(RangeFrom<T>),
    To(RangeToInclusive<T>),
    Inclusive(RangeInclusive<T>),
}

impl<T: 'static + Copy + Eq + Ord + InherentLimits<T>> ValueRange<T> {
    pub fn from_values(start: Option<T>, end: Option<T>) -> Self {
        match (start, end) {
            (Some(start), Some(end)) => Self::Inclusive(start..=end),
            (Some(start), None) => Self::From(start..),
            (None, Some(end)) => Self::To(..=end),
            (None, None) => Self::Full(std::marker::PhantomData),
        }
    }

    pub fn contains(&self, val: T) -> bool {
        match self {
            Self::Full(_) => true,
            Self::From(range) => val >= range.start,
            Self::To(range) => val <= range.end,
            Self::Inclusive(range) => val >= *range.start() && val <= *range.end(),
        }
    }

    pub fn start(&self) -> T {
        match self {
            Self::Full(_) => <T as InherentLimits<T>>::MIN_INT,
            Self::From(range) => range.start,
            Self::To(..) => <T as InherentLimits<T>>::MIN_INT,
            Self::Inclusive(range) => *range.start(),
        }
    }

    pub fn end(&self) -> T {
        match self {
            Self::Full(_) => <T as InherentLimits<T>>::MAX_INT,
            Self::From(..) => <T as InherentLimits<T>>::MAX_INT,
            Self::To(range) => range.end,
            Self::Inclusive(range) => *range.end(),
        }
    }

    pub fn to_std_inclusive_range(
        &self,
        start_override: Option<T>,
        end_override: Option<T>,
    ) -> RangeInclusive<T> {
        match self {
            Self::Full(..) => {
                let start = start_override.unwrap_or_else(|| <T as InherentLimits<T>>::MIN_INT);

                let end = end_override.unwrap_or_else(|| <T as InherentLimits<T>>::MAX_INT);

                start..=end
            }
            Self::From(range) => {
                let start = start_override.unwrap_or_else(|| range.start.clone());

                let end = end_override.unwrap_or_else(|| <T as InherentLimits<T>>::MAX_INT);

                start..=end
            }
            Self::To(range) => {
                let end = end_override.unwrap_or_else(|| range.end.clone());

                let start = start_override.unwrap_or_else(|| <T as InherentLimits<T>>::MIN_INT);

                start..=end
            }
            Self::Inclusive(range) => {
                let start = start_override.unwrap_or_else(|| range.start().clone());
                let end = end_override.unwrap_or_else(|| range.end().clone());

                start..=end
            }
        }
    }
}

pub unsafe trait SoftClamp<T: 'static + Copy + Eq + Ord + InherentLimits<T>>:
    ClampedInteger<T> + InherentBehavior
{
    const VALID_RANGES: &'static [ValueRange<T>];
}

pub unsafe trait HardClamp<T: 'static + Copy + Eq + Ord + InherentLimits<T>>:
    ClampedInteger<T> + InherentBehavior
{
    const VALID_RANGES: &'static [ValueRange<T>];
}

pub unsafe trait ClampedEnum<T: Copy>: ClampedInteger<T> + InherentBehavior {}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum ClampError<T: Copy> {
    #[error("Value too small: {val} (min: {min})")]
    TooSmall { val: T, min: T },
    #[error("Value too large: {val} (max: {max})")]
    TooLarge { val: T, max: T },
    #[error("Value out of bounds")]
    OutOfBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Panicking {}

impl crate::Behavior for Panicking {
    fn add<T: Add<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Add<Output = num::Saturating<T>>,
    {
        let val = lhs + rhs;
        if val > max {
            panic!("Addition overflow");
        }
        if val < min {
            panic!("Addition underflow");
        }
        val
    }

    fn sub<T: Sub<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Sub<Output = num::Saturating<T>>,
    {
        let val = lhs - rhs;
        if val > max {
            panic!("Subtraction overflow");
        }
        if val < min {
            panic!("Subtraction underflow");
        }
        val
    }

    fn mul<T: Mul<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Mul<Output = num::Saturating<T>>,
    {
        let val = lhs * rhs;
        if val > max {
            panic!("Multiplication overflow");
        }
        if val < min {
            panic!("Multiplication underflow");
        }
        val
    }

    fn div<T: Div<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Div<Output = num::Saturating<T>>,
    {
        let val = lhs / rhs;
        if val > max {
            panic!("Division overflow");
        }
        if val < min {
            panic!("Division underflow");
        }
        val
    }

    fn rem<T: Rem<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Rem<Output = num::Saturating<T>>,
    {
        let val = lhs % rhs;
        if val > max {
            panic!("Remainder overflow");
        }
        if val < min {
            panic!("Remainder underflow");
        }
        val
    }

    fn bitand<T: BitAnd<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: BitAnd<Output = num::Saturating<T>>,
    {
        let val = lhs & rhs;
        if val > max {
            panic!("Bitwise AND overflow");
        }
        if val < min {
            panic!("Bitwise AND underflow");
        }
        val
    }

    fn bitor<T: BitOr<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: BitOr<Output = num::Saturating<T>>,
    {
        let val = lhs | rhs;
        if val > max {
            panic!("Bitwise OR overflow");
        }
        if val < min {
            panic!("Bitwise OR underflow");
        }
        val
    }

    fn bitxor<T: BitXor<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: BitXor<Output = num::Saturating<T>>,
    {
        let val = lhs ^ rhs;
        if val > max {
            panic!("Bitwise XOR overflow");
        }
        if val < min {
            panic!("Bitwise XOR underflow");
        }
        val
    }

    // fn shl<T: Shl<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    // where
    //     T::Output: Eq + Ord,
    //     num::Saturating<T>: Shl<Output = num::Saturating<T>>,
    // {
    //     let val = lhs << rhs;
    //     if val > max {
    //         panic!("Bitwise shift left overflow");
    //     }
    //     if val < min {
    //         panic!("Bitwise shift left underflow");
    //     }
    //     val
    // }

    // fn shr<T: Shr<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    // where
    //     T::Output: Eq + Ord,
    //     num::Saturating<T>: Shr<Output = num::Saturating<T>>,
    // {
    //     let val = lhs >> rhs;
    //     if val > max {
    //         panic!("Bitwise shift right overflow");
    //     }
    //     if val < min {
    //         panic!("Bitwise shift right underflow");
    //     }
    //     val
    // }

    fn neg<T: std::ops::Neg<Output = T>>(val: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: std::ops::Neg<Output = num::Saturating<T>>,
    {
        let val = -val;

        if val > max {
            panic!("Negation overflow");
        }
        if val < min {
            panic!("Negation underflow");
        }
        val
    }

    fn not<T: std::ops::Not<Output = T>>(val: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: std::ops::Not<Output = num::Saturating<T>>,
    {
        let val = !val;

        if val > max {
            panic!("Bitwise NOT overflow");
        }
        if val < min {
            panic!("Bitwise NOT underflow");
        }
        val
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Saturating {}

impl crate::Behavior for Saturating {
    fn add<T: Add<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Add<Output = num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs + rhs;
        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }

    fn sub<T: Sub<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Sub<Output = num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs - rhs;
        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }

    fn mul<T: Mul<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Mul<Output = num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs * rhs;
        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }

    fn div<T: Div<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Div<Output = num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs / rhs;
        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }

    fn rem<T: Rem<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Rem<Output = num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs % rhs;
        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }

    fn bitand<T: BitAnd<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: BitAnd<Output = num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs & rhs;
        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }

    fn bitor<T: BitOr<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: BitOr<Output = num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs | rhs;
        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }

    fn bitxor<T: BitXor<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: BitXor<Output = num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs ^ rhs;
        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }

    // fn shl<T: Shl<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    // where
    //     T::Output: Eq + Ord,
    //     num::Saturating<T>: Shl<Output = num::Saturating<T>>,
    // {
    //     let lhs = num::Saturating(lhs);
    //     let rhs = num::Saturating(rhs);
    //     let num::Saturating(val) = lhs << rhs;
    //     if val > max {
    //         max
    //     } else if val < min {
    //         min
    //     } else {
    //         val
    //     }
    // }

    // fn shr<T: Shr<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    // where
    //     T::Output: Eq + Ord,
    //     num::Saturating<T>: Shr<Output = num::Saturating<T>>,
    // {
    //     let lhs = num::Saturating(lhs);
    //     let rhs = num::Saturating(rhs);
    //     let num::Saturating(val) = lhs >> rhs;
    //     if val > max {
    //         max
    //     } else if val < min {
    //         min
    //     } else {
    //         val
    //     }
    // }

    fn neg<T: std::ops::Neg<Output = T>>(val: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: std::ops::Neg<Output = num::Saturating<T>>,
    {
        let val = num::Saturating(val);
        let num::Saturating(val) = -val;

        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }

    fn not<T: std::ops::Not<Output = T>>(val: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: std::ops::Not<Output = num::Saturating<T>>,
    {
        let val = num::Saturating(val);
        let num::Saturating(val) = !val;

        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use checked_rs_macros::clamped;

//     use super::*;
//     use crate::prelude::*;

//     #[test]
//     fn test_define() {
//         #[clamped(u8; default = 1; behavior = Panicking)]
//         #[derive(Debug, Clone, Copy)]
//         pub enum Example {
//             #[eq(0)]
//             Nil,
//             #[eq(u8::MAX)]
//             Invalid,
//             #[rest]
//             Valid,
//         }

//         let a: Example = Default::default();
//         let b: Example = 254.into();
//         let c = a + b;

//         assert!(a.is_valid());
//         assert!(b.is_valid());
//         assert!(c.is_invalid());

//         let d: Example = c - u8::MAX;

//         assert!(d.is_nil());
//     }
// }

use std::{
    num,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Sub},
};

use crate::{InherentBehavior, InherentLimits};
use anyhow::Result;

pub unsafe trait ClampedInteger<T: Copy>:
    'static + Default + Eq + Ord + InherentLimits<T>
{
    fn from_primitive(value: T) -> Result<Self>;
    fn as_primitive(&self) -> &T;

    fn into_primitive(&self) -> T {
        *self.as_primitive()
    }
}

pub unsafe trait SoftClamp<T: Copy>: ClampedInteger<T> + InherentBehavior {}

pub unsafe trait HardClamp<T: Copy>: ClampedInteger<T> + InherentBehavior {}

pub unsafe trait ClampedEnum<T: Copy>: ClampedInteger<T> + InherentBehavior {}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum ClampError<T: Copy> {
    #[error("Value too small: {val} (min: {min})")]
    TooSmall { val: T, min: T },
    #[error("Value too large: {val} (max: {max})")]
    TooLarge { val: T, max: T },
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

    fn neg<T: std::ops::Neg<Output = T>>(value: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: std::ops::Neg<Output = num::Saturating<T>>,
    {
        let val = -value;

        if val > max {
            panic!("Negation overflow");
        }
        if val < min {
            panic!("Negation underflow");
        }
        val
    }

    fn not<T: std::ops::Not<Output = T>>(value: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: std::ops::Not<Output = num::Saturating<T>>,
    {
        let val = !value;

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

    fn neg<T: std::ops::Neg<Output = T>>(value: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: std::ops::Neg<Output = num::Saturating<T>>,
    {
        let value = num::Saturating(value);
        let num::Saturating(val) = -value;

        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }

    fn not<T: std::ops::Not<Output = T>>(value: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: std::ops::Not<Output = num::Saturating<T>>,
    {
        let value = num::Saturating(value);
        let num::Saturating(val) = !value;

        if val > max {
            max
        } else if val < min {
            min
        } else {
            val
        }
    }
}

#[cfg(test)]
mod tests {
    use checked_rs_macros::clamped;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_define() {
        #[clamped(u8; default = 1; behavior = Panicking)]
        #[derive(Debug, Clone, Copy)]
        pub enum Example {
            #[eq(0)]
            Nil,
            #[other]
            Valid,
            #[eq(u8::MAX)]
            Invalid,
        }

        let a: Example = Default::default();
        let b: Example = 254.into();
        let c = a + b;

        assert!(a.is_valid());
        assert!(b.is_valid());
        assert!(c.is_invalid());

        let d: Example = c - u8::MAX;

        assert!(d.is_nil());
    }
}

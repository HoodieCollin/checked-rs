pub mod clamp;
pub mod view;

// pub(crate) use clamp::trait_impl_macro::impl_traits;

mod reexports {
    pub use anyhow::*;
}

pub mod prelude {
    pub use crate::clamp::*;
    pub use crate::reexports::*;
    pub use crate::view::*;
    pub use crate::{Behavior, InherentBehavior, UInteger, UIntegerLimits};
    pub use checked_rs_macros::clamped;
}

mod private {
    #[inline(always)]
    pub const unsafe fn into_u128<T: crate::UInteger>(value: T) -> u128 {
        #[repr(C)]
        union Conversion<U: crate::UInteger> {
            from: U,
            to: u128,
        }

        Conversion { from: value }.to
    }

    #[inline(always)]
    pub const unsafe fn from_u128<T: crate::UInteger>(value: u128) -> T {
        #[repr(C)]
        union Conversion<U: crate::UInteger> {
            from: u128,
            to: U,
        }

        Conversion { from: value }.to
    }

    #[inline(always)]
    pub const fn assert_invariants<T: crate::UInteger, const LOWER: u128, const UPPER: u128>() {
        #[cfg(debug_assertions)]
        {
            assert!(
                LOWER <= UPPER,
                "Lower bound must be less than or equal to upper bound"
            );

            assert!(
                T::MAX >= LOWER,
                "Type maximum must be greater than or equal to lower bound"
            );

            assert!(
                T::MAX >= UPPER,
                "Type maximum must be greater than or equal to upper bound"
            );

            assert!(
                T::MIN <= LOWER,
                "Type minimum must be less than or equal to lower bound"
            );

            assert!(
                T::MIN <= UPPER,
                "Type minimum must be less than or equal to upper bound"
            );
        }
    }

    #[inline(always)]
    pub const fn validate<T: crate::UInteger, const LOWER: u128, const UPPER: u128>(
        value: T,
    ) -> Result<T, crate::clamp::ClampError> {
        let val = unsafe { into_u128(value) };

        if val < LOWER {
            Err(crate::clamp::ClampError::TooSmall { val, min: LOWER })
        } else if val > UPPER {
            Err(crate::clamp::ClampError::TooLarge { val, max: UPPER })
        } else {
            Ok(value)
        }
    }
}

pub trait UIntegerLimits {
    const MIN: u128;
    const MAX: u128;
}

pub trait UInteger:
    UIntegerLimits
    + Copy
    + Default
    + Eq
    + Ord
    + std::ops::Add
    + std::ops::Sub
    + std::ops::Mul
    + std::ops::Div
    + std::ops::Rem
    + std::ops::BitAnd
    + std::ops::BitOr
    + std::ops::BitXor
    + std::ops::Shl
    + std::ops::Shr
{
    fn from_u128(value: u128) -> Self;
    fn into_u128(self) -> u128;
}

macro_rules! impl_unsigned {
    ($($ty:ty),*) => {
        $(
            impl UIntegerLimits for $ty {
                const MAX: u128 = <$ty>::MAX as u128;
                const MIN: u128 = <$ty>::MIN as u128;

            }

            impl UInteger for $ty {
                #[inline(always)]
                fn from_u128(value: u128) -> Self {
                    #[cfg(debug_assertions)]
                    {
                        assert!(value <= <$ty>::MAX as u128, "Value too large for {}", stringify!($ty));
                    }

                    value as Self
                }

                #[inline(always)]
                fn into_u128(self) -> u128 {
                    self as u128
                }
            }
        )*
    };
}

impl_unsigned!(u8, u16, u32, u64, u128);

pub trait Behavior: Copy {
    fn add<T: UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T;
    fn sub<T: UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T;
    fn mul<T: UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T;
    fn div<T: UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T;
    fn rem<T: UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T;
    fn bitand<T: UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T;
    fn bitor<T: UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T;
    fn bitxor<T: UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T;
    fn shl<T: UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T;
    fn shr<T: UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T;
}

pub trait InherentBehavior {
    type Behavior: Behavior;
}

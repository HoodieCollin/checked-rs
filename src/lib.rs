//! # checked-rs
//!
//! > A library for encoding validation semantics into the type system.
//!
//! ## Overview
//!
//! The main components of this library are the structs `View` _(plus the `Validator` trait)_, `HardClamp`, `SoftClamp` and the attribute macro `clamped`.
//! Additionally, there are some traits and types such as `Behavior` and `ClampGuard` that either configure how overflow is handled or provide an alternative way to interact with the clamped types.
//!
//! ### `HardClamp`
//!
//! The `HardClamp` struct is a wrapper around an unsigned integer that clamps the value to a specified range.
//!
//! ```no_run
//! use checked_rs::prelude::*;
//!
//! let mut val = HardClamp::<u8, Saturating, 0, 10>::new(5).unwrap();
//! assert_eq!(val.get(), 5);
//!
//! val += 5;
//! assert_eq!(val.get(), 10);
//!
//! val -= 15;
//! assert_eq!(val.get(), 0);
//!
//! val += 20;
//! assert_eq!(val.get(), 10);
//! ```
//!
//! ### `SoftClamp`
//!
//! The `SoftClamp` struct is a wrapper around an unsigned integer that can be checked for if it is within a specified range.
//!
//! ```no_run
//! use checked_rs::prelude::*;
//!
//! let mut val = SoftClamp::<u8, Saturating, 0, 10>::new(5);
//! assert_eq!(*val, 5);
//! assert_eq!(val.is_valid(), true);
//!
//! val += 5;
//! assert_eq!(*val, 10);
//! assert_eq!(val.is_valid(), true);
//!
//! val -= 15;
//! assert_eq!(*val, 0);
//! assert_eq!(val.is_valid(), true);
//!
//! *val = 30;
//! assert_eq!(*val, 30);
//! assert_eq!(val.is_valid(), false);
//! ```
//!
//! ### `Behavior`
//!
//! The `Behavior` trait is used to configure how overflow is handled for the clamped types.
//! There are two inherent implementations of `Behavior` that can be used: `Panicking` and `Saturating`.
//! The default behavior is to panic on overflow.
//!
//! ### `ClampGuard`
//!
//! The `ClampGuard` struct is a RAII type that is used to modify a clamped value via an exclusive borrow. It will allow the value it tracks to go out of bounds temporarily, but will not allow any value to propagate to the original that is invalid.
//! This is also useful when you want to change a value temporarily and then revert it back to the original value if the change is not valid or is otherwise unwanted.
//!
//! ```no_run
//! use checked_rs::prelude::*;
//!
//! let mut val = HardClamp::<u8, Saturating, 0, 10>::new(5).unwrap();
//!
//! assert_eq!(val.get(), 5);
//!
//! let mut g = val.modify();
//!
//! assert_eq!(g.is_changed(), false);
//!
//! *g = 10;
//!
//! assert_eq!(g.is_changed(), true);
//!
//! g.commit().unwrap();
//!
//! assert_eq!(val.get(), 10);
//!
//! let mut g = val.modify();
//! *g = 15;
//!
//! assert!(g.check().is_err());
//!
//! g.discard();
//!
//! assert_eq!(val.get(), 10);
//! ```
//!
//! ### `View`
//!
//! The `View` struct is a wrapper around a value that encodes it's validation logic into the wrapper. The `Validator` trait is used to define the validation logic for a `View`.
//! This wrapper is lightweight and can be used in place of the raw value via the `Deref` and/or  `AsRef` traits.
//!
//! ```no_run
//! use checked_rs::prelude::*;
//!
//! #[derive(Clone, Copy)]
//! struct NotSeven;
//!
//! impl Validator for NotSeven {
//!     type Item = i32;
//!     type Error = anyhow::Error;
//!
//!     fn validate(item: &Self::Item) -> Result<()> {
//!         if *item == 7 {
//!             Err(anyhow::anyhow!("Value must not be 7"))
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! let mut item = View::with_validator(0, NotSeven);
//! let mut g = item.modify();
//!
//! *g = 7;
//! assert_eq!(*g, 7);
//! assert!(g.check().is_err());
//!
//! *g = 10;
//! assert!(g.commit().is_ok());
//!
//! // the guard is consumed by commit, so we can't check it again
//! // the `View`'s value should be updated
//! assert_eq!(&*item, &10);
//!
//! ```
//!
//! ### `clamped` attribute macro
//!
//! The `clamped` attribute macro is used to create a specialized clamped type. The macro can only be used on enums where each variant represents a specific state within the clamped range.
//! This can be useful when there are multiple states that correspond to a certain set of rules and you want them to be easily distinguishable while still being able to use them in a single integer-like type.
//!
//! ```ignore
//! use checked_rs::prelude::*;
//!
//! #[clamped(u16, default = 600, behavior = Saturating, lower = 100, upper = 600)]
//! #[derive(Debug, Clone, Copy)]
//! enum ResponseCode {
//!     #[eq(100)]
//!     Continue,
//!     #[eq(200)]
//!     Success,
//!     #[eq(300)]
//!     Redirection,
//!     #[eq(400)]
//!     BadRequest,
//!     #[eq(404)]
//!     NotFound,
//!     #[range(500..=599)]
//!     ServerError,
//!     #[other]
//!     Unknown,
//!     #[eq(600)]
//!     Invalid,
//! }
pub mod clamp;
pub mod guard;
pub mod view;

mod reexports {
    #[doc(hidden)]
    pub use anyhow::*;
    #[doc(hidden)]
    pub use serde;
}

pub mod prelude {
    pub use crate::reexports::*;

    pub use crate::clamp::*;
    pub use crate::commit_or_bail;
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
    'static
    + UIntegerLimits
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[clamped(u16, default = 600, behavior = Saturating, lower = 100, upper = 600)]
    #[derive(Debug, Clone, Copy)]
    enum ResponseCode {
        #[eq(100)]
        Continue,
        #[eq(200)]
        Success,
        #[eq(300)]
        Redirection,
        #[eq(400)]
        BadRequest,
        #[eq(404)]
        NotFound,
        #[range(500..=599)]
        ServerError,
        #[other]
        Unknown,
        #[eq(600)]
        Invalid,
    }

    #[test]
    fn test_response_code() {
        let mut code = ResponseCode::new_success();
        assert!(code.is_success());

        code += 100;
        assert!(code.is_redirection());

        code += u16::MAX;
        assert!(code.is_invalid());

        code -= 10;
        assert!(code.is_server_error());

        let mut g = code.modify();

        *g = 111;

        assert!(g.commit().is_ok());
        assert!(code.is_unknown());
    }
}

//! # checked-rs
//!
//! > A library for encoding validation semantics into the type system.
//!
//! ## Overview
//!
//! The main components of this library is the the attribute macro `clamped` and the `View` struct _(plus the `Validator` trait)_.
//! Additionally, there are some traits and types such as `Behavior` and `ClampGuard` that either configure how overflow is handled or provide an alternative way to interact with the clamped types.
//!
//! ### `clamped` attribute macro
//!
//! The `clamped` attribute macro is used to create a specialized clamped type. The macro can be used on either field-less structs or enums with field-less variants.
//! Whe used on a struct, the struct will be transformed to have a single field that is the clamped value. When used on an enum, the enum will be transformed to have a variant for each state within the clamped range.
//!
//! > For the remainder of these docs, `int` will be used to refer to the integer type used for the clamped value.
//!
//! The macro requires the following positional arguments:
//! - `int`: The integer type to use for the clamped value.
//!
//! The macro accepts the following arguments _(in any order)_:
//! - `behavior`: The behavior to use when the value overflows the limits. The default behavior is `Panicking`.
//! - `default`: The default value to use when the value is not provided. The default default value is zero _(if possible)_ or the minimum value.
//! - `lower`: The lower limit of the clamped value. The default lower limit is the minimum value of `int`.
//! - `upper`: The upper limit of the clamped value. The default upper limit is the maximum value of `int`.
//!
//! The transformed type will have the following inherent implementations:
//! - `new(value: int) -> Self`: A constructor that creates a new clamped value from the provided value.
//! - `rand() -> Self`: A method that generates a random value within the clamped range.
//! - `validate(value: int) -> Result<int, Error>`: A method that validates the provided value and returns the value if it is within the clamped range.
//! - `modify<'a>(&'a mut self) -> Guard<'a>`: A method that returns a guard that can be used to stage _(potentially out-of-bounds)_ changes to the clamped value and either commit or discard the changes.
//!
//! The transformed type will have the following custom traits implemented:
//! - `InherentLimits<int>`: A trait that defines the minimum and maximum values of the clamped range.
//! - `InherentBehavior`: A trait that defines the behavior to use when the value overflows the limits.
//! - `ClampedInteger<int>`: A trait that defines the methods for converting to and from `int`.
//!
//! The transformed type will have the following standard traits implemented:
//! - `Default`, `Deref`, `AsRef`, `FromStr`, `PartialEq`, `PartialOrd`, `Eq`, `Ord`, `Add`, `AddAssign`, `Sub`, `SubAssign`, `Mul`, `MulAssign`, `Div`, `DivAssign`, `Rem`, `RemAssign`, `Neg`, `Not`, `BitAnd`, `BitAndAssign`, `BitOr`, `BitOrAssign`, `BitXor`, `BitXorAssign`.
//! - `From` implementations are provided to support conversions for the same machine integer types as `int`.
//!
//! > **NOTE**: The `std::cmp` and `std::ops` traits support `rhs` values of the clamped type or `int`.
//!
//! The transformed type will have the following external traits implemented:
//! - `serde::Serialize`, `serde::Deserialize`
//!
//! ### Struct Usage
//!
//! When used on a struct, you can optionally specify if it should be a `Soft` or `Hard` clamped type.
//!
//! #### Soft Clamps
//!
//! Soft clamps are clamped types that **_DO NOT_** enforce the limits on the value. Instead, the value is clamped when it is assigned via the `set` method. The method `set_unchecked` can be used to set the value without clamping. Alternatively, the method `get_mut` can be used to get a mutable reference to the inner value or the arithmetic traits can be used to perform operations on the value without clamping.
//!
//! Additionally, they will have the following extra standard traits implemented:
//! - `DerefMut`, `AsMut`
//!
//! ```ignore
//! use checked_rs::prelude::*;
//!
//! #[clamped(i32, lower = -100_000, upper = 100_000)]
//! #[derive(Debug, Clone, Copy, Hash)]
//! struct Scale;
//! ```
//!
//! #### Hard Clamps
//!
//! Hard clamps are clamped types that **_DO_** enforce the limits on the value. The value is clamped when it is created and any operations that would cause the value to overflow the limits will be handled according to the specified behavior.
//!
//! > **UNSAFE NOTE**: The `set_unchecked` and `as_mut` methods are available but marked unsafe because they can be used to assign an out-of-bounds value.
//!
//! ```ignore
//! use checked_rs::prelude::*;
//!
//! #[clamped(usize, default = 1_000, upper = 100_000_000)]
//! #[derive(Debug, Clone, Copy, Hash)]
//! struct Available;
//! ```
//!
//! ### Enum Usage
//!
//! Each variant of the enum will either represent a specific value within the overall clamped range, a hard clamped sub-range or a special variant that represents any value that is not explicitly handled. The variants will have corresponding methods that can be used to create a new instances of that variant or check if the contained value is that variant.
//!
//! > **NOTE**: The enum must account for all possible values within the clamped range. This can be done by using the `#[eq]` and `#[range]` attributes on the variants.
//! > The `#[other]` attribute can be used to account for any values that are not explicitly handled.
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
//!
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

use std::{
    num,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Sub},
};
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
    };

    pub use checked_rs_macros::clamped;
}

pub trait Behavior: Copy + 'static {
    // Binary Ops
    fn add<T: Add<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Add<Output = num::Saturating<T>>;
    fn sub<T: Sub<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Sub<Output = num::Saturating<T>>;
    fn mul<T: Mul<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Mul<Output = num::Saturating<T>>;
    fn div<T: Div<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Div<Output = num::Saturating<T>>;
    fn rem<T: Rem<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: Rem<Output = num::Saturating<T>>;
    fn bitand<T: BitAnd<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: BitAnd<Output = num::Saturating<T>>;
    fn bitor<T: BitOr<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: BitOr<Output = num::Saturating<T>>;
    fn bitxor<T: BitXor<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: BitXor<Output = num::Saturating<T>>;
    // fn shl<T: Shl<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    // where
    //     T::Output: Eq + Ord,
    //     num::Saturating<T>: Shl<Output = num::Saturating<T>>;
    // fn shr<T: Shr<Output = T>>(lhs: T, rhs: T, min: T::Output, max: T::Output) -> T::Output
    // where
    //     T::Output: Eq + Ord,
    //     num::Saturating<T>: Shr<Output = num::Saturating<T>>;
    // Unary Ops
    fn neg<T: std::ops::Neg<Output = T>>(value: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: std::ops::Neg<Output = num::Saturating<T>>;
    fn not<T: std::ops::Not<Output = T>>(value: T, min: T::Output, max: T::Output) -> T::Output
    where
        T::Output: Eq + Ord,
        num::Saturating<T>: std::ops::Not<Output = num::Saturating<T>>;
}

pub trait InherentLimits<T>: 'static {
    const MIN: Self;
    const MAX: Self;
    const MIN_INT: T;
    const MAX_INT: T;
}

pub trait InherentBehavior: 'static {
    type Behavior: Behavior;
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn test_enum_simple() {
    //     clamped! {
    //         #[usize]
    //         enum DoubleSentinel {
    //             Zero(0),
    //             Valid(..),
    //             Invalid(usize::MAX),
    //         }
    //     }
    // }

    // #[test]
    // fn test_enum_non_comprehensive() {
    //     clamped! {
    //         #[usize]
    //         enum DoubleSentinel {
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

    // #[test]
    // fn test_enum_nested() {
    //     clamped! {
    //         #[usize]
    //         enum ResponseCode {
    //             Success[200..300] {
    //                 Okay(200),
    //                 Created(201),
    //                 Accepted(202),
    //                 Unknown(..),
    //             },
    //             Error {
    //                 Client[400..500] {
    //                     BadRequest(400),
    //                     Unauthorized(401),
    //                     PaymentRequired(402),
    //                     Forbidden(403),
    //                     NotFound(404),
    //                     Unknown(..)
    //                 },
    //                 Server[500..600] {
    //                     Internal(500),
    //                     NotImplemented(501),
    //                     BadGateway(502),
    //                     ServiceUnavailable(503),
    //                     GatewayTimeout(504),
    //                     Unknown(..)
    //                 }
    //             }
    //         }
    //     }
    // }

    // #[test]
    // fn test_struct_soft() {
    //     clamped! {
    //         #[usize as Soft]
    //         struct TenOrLess(..=10);
    //     }
    // }

    // #[test]
    // fn test_struct_hard() {
    //     clamped! {
    //         #[usize as Hard]
    //         struct TenOrMore(10..);
    //     }
    // }

    // #[test]
    // fn test_struct_multiple_ranges() {
    //     clamped! {
    //         #[usize as Hard]
    //         struct LessThanTenOrBetween999And2000(..10, 1000..2000);
    //     }
    // }
}

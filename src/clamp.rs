use std::{
    num,
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, RangeInclusive, Rem, Sub},
};

use crate::{InherentBehavior, InherentLimits, OpBehaviorParams};
use anyhow::Result;

pub unsafe trait ClampedInteger<T: Copy>:
    'static + Default + Eq + Ord + InherentLimits<T>
{
    fn from_primitive(val: T) -> Result<Self>;

    unsafe fn from_primitive_unchecked(val: T) -> Self {
        Self::from_primitive(val).unwrap_unchecked()
    }

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

                #[inline(always)]
                fn is_zero(&self) -> bool {
                    *self == 0
                }

                #[inline(always)]
                #[allow(unused_comparisons)]
                fn is_negative(&self) -> bool {
                    *self < 0
                }

                #[inline(always)]
                fn is_positive(&self) -> bool {
                    *self > 0
                }
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

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct ValueRangeInclusive<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
    pub RangeInclusive<T>,
);

impl<T: 'static + Copy + Eq + Ord + InherentLimits<T>> ValueRangeInclusive<T> {
    pub fn contains(&self, val: T) -> bool {
        val >= *self.0.start() && val <= *self.0.end()
    }

    pub fn first_val(&self) -> T {
        *self.0.start()
    }

    pub fn last_val(&self) -> T {
        *self.0.end()
    }
}

/// # Invariants
/// - `VALUES` must be sorted in ascending order
pub unsafe trait ExactValues<T: 'static + Copy + Eq + Ord>:
    'static + Default + Eq + Ord
{
    const VALUES: &'static [T];

    fn contains_value(val: T) -> bool {
        Self::VALUES.contains(&val)
    }
}

/// # Invariants
/// - `VALID_RANGES` must be sorted in ascending order
pub unsafe trait RangeValues<T: 'static + Copy + Eq + Ord + InherentLimits<T>>:
    ClampedInteger<T> + InherentBehavior
{
    const VALID_RANGES: &'static [ValueRangeInclusive<T>];
}

pub unsafe trait SoftClamp<T: 'static + Copy + Eq + Ord + InherentLimits<T>>:
    RangeValues<T>
{
}

pub unsafe trait HardClamp<T: 'static + Copy + Eq + Ord + InherentLimits<T>>:
    RangeValues<T>
{
}

pub unsafe trait ClampedEnum<T: Copy>: ClampedInteger<T> + InherentBehavior {}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum ClampError<T: Copy> {
    #[error("Value too small: {val} (min: {min})")]
    TooSmall { val: T, min: T },
    #[error("Value too large: {val} (max: {max})")]
    TooLarge { val: T, max: T },
    #[error("Value out of bounds: {val} (between ranges: {left_min}..={left_max} and {right_min}..={right_max})")]
    OutOfBounds {
        val: T,
        left_min: T,
        left_max: T,
        right_min: T,
        right_max: T,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Panicking {}

fn maybe_panic<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
    op_name: &str,
    val: T,
    params: OpBehaviorParams<T>,
) -> T {
    match params {
        OpBehaviorParams::Simple { min, max } => {
            if val < min {
                panic!("{} underflow", op_name);
            }

            if val > max {
                panic!("{} overflow", op_name);
            }

            return val;
        }
        OpBehaviorParams::ExactsOnly(exacts) => {
            for exact in exacts {
                if val == *exact {
                    return val;
                }
            }

            panic!("{} result is not an allowed exact value", op_name);
        }
        OpBehaviorParams::RangesOnly(ranges) => {
            for range in ranges {
                if range.contains(val) {
                    return val;
                }
            }

            panic!("{} result is out of bounds", op_name);
        }
        OpBehaviorParams::ExactsAndRanges { exacts, ranges } => {
            for exact in exacts {
                if val == *exact {
                    return val;
                }
            }

            for range in ranges {
                if range.contains(val) {
                    return val;
                }
            }

            panic!("{} result is out of bounds", op_name);
        }
    }
}

impl crate::Behavior for Panicking {
    fn add<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Add<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Add<Output = num::Saturating<T>>,
        <num::Saturating<T> as Add>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        maybe_panic("Addition", lhs + rhs, params)
    }

    fn sub<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Sub<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Sub<Output = num::Saturating<T>>,
        <num::Saturating<T> as Sub>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        maybe_panic("Subtraction", lhs - rhs, params)
    }

    fn mul<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Mul<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Mul<Output = num::Saturating<T>>,
        <num::Saturating<T> as Mul>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        maybe_panic("Multiplication", lhs * rhs, params)
    }

    fn div<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Div<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Div<Output = num::Saturating<T>>,
        <num::Saturating<T> as Div>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        maybe_panic("Division", lhs / rhs, params)
    }

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
        <num::Saturating<T> as Rem>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        maybe_panic("Remainder", lhs % rhs, params)
    }

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
        <num::Saturating<T> as BitAnd>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        maybe_panic("Bitwise AND", lhs & rhs, params)
    }

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
        <num::Saturating<T> as BitOr>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        maybe_panic("Bitwise OR", lhs | rhs, params)
    }

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
        <num::Saturating<T> as BitXor>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        maybe_panic("Bitwise XOR", lhs ^ rhs, params)
    }

    fn neg<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        val: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Neg<Output = T> + Sub<Output = T>,
        <T as Neg>::Output: Eq + Ord + Into<T>,
        <T as Sub>::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Neg<Output = num::Saturating<T>>,
        <num::Saturating<T> as Neg>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        maybe_panic("Negation", -val, params)
    }

    fn not<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        val: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Not<Output = T> + Sub<Output = T>,
        <T as Not>::Output: Eq + Ord + Into<T>,
        <T as Sub>::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Not<Output = num::Saturating<T>>,
        <num::Saturating<T> as Not>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        maybe_panic("Bitwise NOT", !val, params)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Saturating {}

fn left_saturating_exacts<T: Copy + Eq + Ord + InherentLimits<T>>(
    val: T,
    exacts: &[T],
) -> Option<T> {
    for (left, right) in exacts.windows(2).map(|w| (w[0], w[1])) {
        if val == left || val == right {
            return Some(val);
        }

        if val > left && val < right {
            // val is in the middle of two exact values
            return Some(left);
        }
    }

    None
}

fn right_saturating_exacts<T: Copy + Eq + Ord + InherentLimits<T>>(
    val: T,
    exacts: &[T],
) -> Option<T> {
    for (left, right) in exacts.windows(2).map(|w| (w[0], w[1])) {
        if val == left || val == right {
            return Some(val);
        }

        if val > left && val < right {
            // val is in the middle of two exact values
            return Some(right);
        }
    }

    None
}

fn nearest_saturating_exacts<T: Copy + Eq + Ord + InherentLimits<T> + Sub<Output = T>>(
    val: T,
    exacts: &[T],
) -> Option<T> {
    for (left, right) in exacts.windows(2).map(|w| (w[0], w[1])) {
        if val == left || val == right {
            return Some(val);
        }

        if val > left && val < right {
            // val is in the middle of two exact values
            let left_diff = val - left;
            let right_diff = right - val;

            if left_diff < right_diff {
                return Some(left);
            } else {
                return Some(right);
            }
        }
    }

    None
}

fn left_saturating_ranges<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
    val: T,
    ranges: &[ValueRangeInclusive<T>],
) -> Option<T> {
    for (left, right) in ranges.windows(2).map(|w| (&w[0], &w[1])) {
        if left.contains(val) {
            return Some(val);
        }

        if val > left.last_val() && val < right.first_val() {
            // val is in the middle of two ranges
            return Some(left.last_val());
        }
    }

    None
}

fn right_saturating_ranges<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
    val: T,
    ranges: &[ValueRangeInclusive<T>],
) -> Option<T> {
    for (left, right) in ranges.windows(2).map(|w| (&w[0], &w[1])) {
        if left.contains(val) {
            return Some(val);
        }

        if val > left.last_val() && val < right.first_val() {
            // val is in the middle of two ranges
            return Some(right.first_val());
        }
    }

    None
}

fn nearest_saturating_ranges<T: 'static + Copy + Eq + Ord + InherentLimits<T> + Sub<Output = T>>(
    val: T,
    ranges: &[ValueRangeInclusive<T>],
) -> Option<T> {
    for (left, right) in ranges.windows(2).map(|w| (&w[0], &w[1])) {
        if left.contains(val) {
            return Some(val);
        }

        if val > left.last_val() && val < right.first_val() {
            // val is in the middle of two ranges
            let left_diff = val - left.last_val();
            let right_diff = right.first_val() - val;

            if left_diff < right_diff {
                return Some(left.last_val());
            } else {
                return Some(right.first_val());
            }
        }
    }

    None
}

#[inline(always)]
fn resolve_saturation_left<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
    val: T,
    params: OpBehaviorParams<T>,
) -> T {
    match params {
        OpBehaviorParams::Simple { min, max } => {
            if val < min {
                min
            } else if val > max {
                max
            } else {
                val
            }
        }
        OpBehaviorParams::ExactsOnly(exacts) => {
            #[cfg(debug_assertions)]
            {
                if exacts.len() == 0 {
                    panic!("No values provided");
                }
            }

            if let Some(val) = left_saturating_exacts(val, exacts) {
                val
            } else if val < exacts[0] {
                exacts[0]
            } else {
                exacts[exacts.len() - 1]
            }
        }
        OpBehaviorParams::RangesOnly(ranges) => {
            #[cfg(debug_assertions)]
            {
                if ranges.len() == 0 {
                    panic!("No ranges provided");
                }
            }

            if let Some(val) = left_saturating_ranges(val, ranges) {
                return val;
            }

            let lower_limit = ranges[0].first_val();
            let upper_limit = ranges[ranges.len() - 1].last_val();

            if val < lower_limit {
                lower_limit
            } else {
                upper_limit
            }
        }
        OpBehaviorParams::ExactsAndRanges { exacts, ranges } => {
            #[cfg(debug_assertions)]
            {
                if exacts.len() == 0 {
                    panic!("No values provided");
                }
            }

            #[cfg(debug_assertions)]
            {
                if exacts.len() == 0 {
                    panic!("No ranges provided");
                }
            }

            if let Some(val) = left_saturating_exacts(val, exacts) {
                return val;
            }

            if let Some(val) = left_saturating_ranges(val, ranges) {
                return val;
            }

            let lower_limit = exacts[0].min(ranges[0].first_val());
            let upper_limit = exacts[exacts.len() - 1].max(ranges[ranges.len() - 1].last_val());

            if val < lower_limit {
                lower_limit
            } else {
                upper_limit
            }
        }
    }
}

#[inline(always)]
fn resolve_saturation_right<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
    val: T,
    params: OpBehaviorParams<T>,
) -> T {
    match params {
        OpBehaviorParams::Simple { min, max } => {
            if val < min {
                min
            } else if val > max {
                max
            } else {
                val
            }
        }
        OpBehaviorParams::ExactsOnly(exacts) => {
            #[cfg(debug_assertions)]
            {
                if exacts.len() == 0 {
                    panic!("No values provided");
                }
            }

            if let Some(val) = right_saturating_exacts(val, exacts) {
                val
            } else if val < exacts[0] {
                exacts[0]
            } else {
                exacts[exacts.len() - 1]
            }
        }
        OpBehaviorParams::RangesOnly(ranges) => {
            #[cfg(debug_assertions)]
            {
                if ranges.len() == 0 {
                    panic!("No ranges provided");
                }
            }

            if let Some(val) = right_saturating_ranges(val, ranges) {
                return val;
            }

            let lower_limit = ranges[0].first_val();
            let upper_limit = ranges[ranges.len() - 1].last_val();

            if val < lower_limit {
                lower_limit
            } else {
                upper_limit
            }
        }
        OpBehaviorParams::ExactsAndRanges { exacts, ranges } => {
            #[cfg(debug_assertions)]
            {
                if exacts.len() == 0 {
                    panic!("No values provided");
                }
            }

            #[cfg(debug_assertions)]
            {
                if exacts.len() == 0 {
                    panic!("No ranges provided");
                }
            }

            if let Some(val) = right_saturating_exacts(val, exacts) {
                return val;
            }

            if let Some(val) = right_saturating_ranges(val, ranges) {
                return val;
            }

            let lower_limit = exacts[0].min(ranges[0].first_val());
            let upper_limit = exacts[exacts.len() - 1].max(ranges[ranges.len() - 1].last_val());

            if val < lower_limit {
                lower_limit
            } else {
                upper_limit
            }
        }
    }
}

#[inline(always)]
fn resolve_saturation_nearest<
    T: 'static + Copy + Eq + Ord + InherentLimits<T> + Sub<Output = T>,
>(
    val: T,
    params: OpBehaviorParams<T>,
) -> T {
    match params {
        OpBehaviorParams::Simple { min, max } => {
            if val < min {
                min
            } else if val > max {
                max
            } else {
                val
            }
        }
        OpBehaviorParams::ExactsOnly(exacts) => {
            #[cfg(debug_assertions)]
            {
                if exacts.len() == 0 {
                    panic!("No values provided");
                }
            }

            if let Some(val) = nearest_saturating_exacts(val, exacts) {
                val
            } else if val < exacts[0] {
                exacts[0]
            } else {
                exacts[exacts.len() - 1]
            }
        }
        OpBehaviorParams::RangesOnly(ranges) => {
            #[cfg(debug_assertions)]
            {
                if ranges.len() == 0 {
                    panic!("No ranges provided");
                }
            }

            if let Some(val) = nearest_saturating_ranges(val, ranges) {
                return val;
            }

            let lower_limit = ranges[0].first_val();
            let upper_limit = ranges[ranges.len() - 1].last_val();

            if val < lower_limit {
                lower_limit
            } else {
                upper_limit
            }
        }
        OpBehaviorParams::ExactsAndRanges { exacts, ranges } => {
            #[cfg(debug_assertions)]
            {
                if exacts.len() == 0 {
                    panic!("No values provided");
                }
            }

            #[cfg(debug_assertions)]
            {
                if exacts.len() == 0 {
                    panic!("No ranges provided");
                }
            }

            if let Some(val) = nearest_saturating_exacts(val, exacts) {
                return val;
            }

            if let Some(val) = nearest_saturating_ranges(val, ranges) {
                return val;
            }

            let lower_limit = exacts[0].min(ranges[0].first_val());
            let upper_limit = exacts[exacts.len() - 1].max(ranges[ranges.len() - 1].last_val());

            if val < lower_limit {
                lower_limit
            } else {
                upper_limit
            }
        }
    }
}

impl crate::Behavior for Saturating {
    fn add<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Add<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Add<Output = num::Saturating<T>>,
        <num::Saturating<T> as Add>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs + rhs;

        resolve_saturation_left(val, params)
    }

    fn sub<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Sub<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Sub<Output = num::Saturating<T>>,
        <num::Saturating<T> as Sub>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs - rhs;

        resolve_saturation_right(val, params)
    }

    fn mul<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Mul<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Mul<Output = num::Saturating<T>>,
        <num::Saturating<T> as Mul>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs * rhs;

        resolve_saturation_left(val, params)
    }

    fn div<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        lhs: T,
        rhs: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Div<Output = T>,
        T::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Div<Output = num::Saturating<T>>,
        <num::Saturating<T> as Div>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs / rhs;

        resolve_saturation_right(val, params)
    }

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
        <num::Saturating<T> as Rem>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs % rhs;

        resolve_saturation_nearest(val, params)
    }

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
        <num::Saturating<T> as BitAnd>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs & rhs;

        resolve_saturation_nearest(val, params)
    }

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
        <num::Saturating<T> as BitOr>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs | rhs;

        resolve_saturation_nearest(val, params)
    }

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
        <num::Saturating<T> as BitXor>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        let lhs = num::Saturating(lhs);
        let rhs = num::Saturating(rhs);
        let num::Saturating(val) = lhs ^ rhs;

        resolve_saturation_nearest(val, params)
    }

    fn neg<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        val: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Neg<Output = T> + Sub<Output = T>,
        <T as Neg>::Output: Eq + Ord + Into<T>,
        <T as Sub>::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Neg<Output = num::Saturating<T>>,
        <num::Saturating<T> as Neg>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        let val = num::Saturating(val);
        let num::Saturating(val) = -val;

        if <T as InherentLimits<T>>::is_zero(&val) {
            resolve_saturation_nearest(val, params)
        } else if <T as InherentLimits<T>>::is_negative(&val) {
            resolve_saturation_right(val, params)
        } else {
            resolve_saturation_left(val, params)
        }
    }

    fn not<T: 'static + Copy + Eq + Ord + InherentLimits<T>>(
        val: T,
        params: OpBehaviorParams<T>,
    ) -> T
    where
        T: Not<Output = T> + Sub<Output = T>,
        <T as Not>::Output: Eq + Ord + Into<T>,
        <T as Sub>::Output: Eq + Ord + Into<T>,
        num::Saturating<T>: Not<Output = num::Saturating<T>>,
        <num::Saturating<T> as Not>::Output: Eq + Ord + Into<num::Saturating<T>>,
    {
        let val = num::Saturating(val);
        let num::Saturating(val) = !val;

        if <T as InherentLimits<T>>::is_zero(&val) {
            resolve_saturation_nearest(val, params)
        } else if <T as InherentLimits<T>>::is_negative(&val) {
            resolve_saturation_right(val, params)
        } else {
            resolve_saturation_left(val, params)
        }
    }
}

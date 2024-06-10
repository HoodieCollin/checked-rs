use rand::Rng;

use crate::{
    clamp::{ClampError, ClampValidator},
    guard::Guard,
    private, Behavior, UInteger,
};

#[derive(Debug, Clone, Copy, checked_rs_macros::CheckedRsOps)]
#[derive_deref_mut]
#[repr(transparent)]
pub struct SoftClamp<T: UInteger, B: Behavior, const L: u128, const U: u128>(
    pub(crate) T,
    pub(crate) std::marker::PhantomData<B>,
);

impl<T: UInteger, B: Behavior, const L: u128, const U: u128> SoftClamp<T, B, L, U> {
    pub const MIN: T = unsafe { private::from_u128(L) };
    pub const MAX: T = unsafe { private::from_u128(U) };

    pub(crate) const fn assert_invariants() {
        private::assert_invariants::<T, L, U>();
    }

    #[inline(always)]
    pub const fn new(value: T) -> Self {
        Self::assert_invariants();

        Self(value, std::marker::PhantomData)
    }

    #[inline(always)]
    pub const fn new_unchecked(value: T) -> Self {
        Self(value, std::marker::PhantomData)
    }

    #[inline(always)]
    pub const fn validate(value: T) -> Result<T, ClampError> {
        Self::assert_invariants();

        private::validate::<T, L, U>(value)
    }

    #[inline(always)]
    pub fn rand() -> Self {
        Self::assert_invariants();

        Self(
            T::from_u128(rand::thread_rng().gen_range(L..=U)),
            std::marker::PhantomData,
        )
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        Self::validate(self.0).is_ok()
    }

    #[inline(always)]
    pub fn set(&mut self, value: T) -> Result<(), ClampError> {
        Self::validate(value)?;
        self.0 = value;
        Ok(())
    }

    #[inline(always)]
    pub fn set_unchecked(&mut self, value: T) {
        self.0 = value;
    }

    #[inline(always)]
    pub fn get(&self) -> Result<T, ClampError> {
        Self::validate(self.0)
    }

    #[inline(always)]
    pub fn get_unchecked(&self) -> T {
        self.0
    }

    #[inline(always)]
    pub unsafe fn get_mut_unchecked(&mut self) -> &mut T {
        &mut self.0
    }

    #[inline(always)]
    pub fn modify<'a>(&'a mut self) -> Guard<'a, T, ClampError, ClampValidator<T, L, U>> {
        Guard::new(&mut self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::Saturating, *};
    use anyhow::Result;

    #[test]
    fn test_arithmetic() -> Result<()> {
        let mut clamp = SoftClamp::<u8, Saturating, 0, 10>::new(5);
        assert_eq!(*clamp, 5);
        assert!(clamp.is_valid());

        clamp += 5;
        assert_eq!(*clamp, 10);
        assert!(clamp.is_valid());

        clamp -= 15;
        assert_eq!(*clamp, 0);
        assert!(clamp.is_valid());

        *clamp = 30;
        assert_eq!(*clamp, 30);
        assert_eq!(clamp.is_valid(), false);

        Ok(())
    }
}

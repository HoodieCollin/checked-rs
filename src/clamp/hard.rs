use rand::Rng;

use crate::{
    clamp::{guard::ClampGuard, ClampError},
    private, Behavior, UInteger,
};

#[derive(Debug, Clone, Copy, checked_rs_macros::CheckedRsOps)]
#[repr(transparent)]
pub struct HardClamp<T: UInteger, B: Behavior, const L: u128, const U: u128>(
    pub(crate) T,
    pub(crate) std::marker::PhantomData<B>,
);

// inherent methods
impl<T: UInteger, Panicking, const L: u128, const U: u128> HardClamp<T, Panicking, L, U>
where
    Panicking: Behavior,
{
    #[inline(always)]
    pub fn new_or_panic(value: T) -> Self {
        Self::assert_invariants();

        Self(Self::validate(value).unwrap(), std::marker::PhantomData)
    }
}

impl<T: UInteger, Saturating, const L: u128, const U: u128> HardClamp<T, Saturating, L, U>
where
    Saturating: Behavior,
{
    #[inline(always)]
    pub fn new_valid(value: T) -> Self {
        Self::assert_invariants();

        let at_least = std::cmp::max(value.into_u128(), L);
        let at_most = std::cmp::min(at_least, U);

        unsafe { Self::new_unchecked(private::from_u128(at_most)) }
    }
}

impl<T: UInteger, B: Behavior, const L: u128, const U: u128> HardClamp<T, B, L, U> {
    pub const MIN: T = unsafe { private::from_u128(L) };
    pub const MAX: T = unsafe { private::from_u128(U) };

    #[inline(always)]
    pub(crate) const fn assert_invariants() {
        private::assert_invariants::<T, L, U>();
    }

    #[inline(always)]
    pub fn new(value: T) -> Result<Self, ClampError> {
        Self::assert_invariants();

        Self::validate(value).map(|_| Self(value, std::marker::PhantomData))
    }

    #[inline(always)]
    pub const unsafe fn new_unchecked(value: T) -> Self {
        Self::assert_invariants();

        Self(value, std::marker::PhantomData)
    }

    #[inline(always)]
    pub const fn validate(value: T) -> Result<T, ClampError> {
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
    pub fn set(&mut self, value: T) -> Result<(), ClampError> {
        Self::validate(value)?;
        self.0 = value;
        Ok(())
    }

    #[inline(always)]
    pub unsafe fn set_unchecked(&mut self, value: T) {
        self.0 = value;
    }

    #[inline(always)]
    pub fn get(&self) -> T {
        self.0
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
    pub fn modify<'a>(&'a mut self) -> ClampGuard<'a, T, B, L, U> {
        ClampGuard::new(self.get(), unsafe { self.get_mut_unchecked() })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::clamp::{guard::GuardState, hard::HardClamp, Saturating};

    #[test]
    fn test_arithmetic() -> Result<()> {
        let mut clamp = HardClamp::<u8, Saturating, 0, 10>::new(5)?;

        assert_eq!(clamp.get(), 5);

        clamp += 5;

        assert_eq!(clamp.get(), 10);

        clamp -= 15;

        assert_eq!(clamp.get(), 0);

        clamp += 20;

        assert_eq!(clamp.get(), 10);

        clamp /= 2;

        assert_eq!(clamp.get(), 5);

        clamp *= 2;

        assert_eq!(clamp.get(), 10);

        clamp *= 2;

        assert_eq!(clamp.get(), 10);

        clamp %= 2;

        assert_eq!(clamp.get(), 0);

        Ok(())
    }

    #[test]
    fn test_strict_clamp_guard() -> Result<()> {
        let mut clamp = HardClamp::<u8, Saturating, 0, 10>::new(5)?;

        assert_eq!(clamp.get(), 5);

        let mut g = clamp.modify();

        assert_eq!(g.check(), GuardState::Unchanged);

        *g = 10;

        assert_eq!(g.check(), GuardState::Changed);

        g.commit()?;

        assert_eq!(clamp.get(), 10);

        let mut g = clamp.modify();

        assert_eq!(g.check(), GuardState::Unchanged);

        *g = 15;

        assert_eq!(g.check(), GuardState::Changed);

        assert!(g.commit().is_err());

        assert_eq!(clamp.get(), 10);

        Ok(())
    }
}

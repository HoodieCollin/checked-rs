use crate::{clamp::ClampError, private, Behavior, UInteger};

#[derive(Debug)]
pub struct ClampGuard<'a, T: UInteger, B: Behavior, const L: u128, const U: u128>(
    pub(crate) T,
    pub(crate) &'a mut T,
    pub(crate) std::marker::PhantomData<B>,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardState {
    Changed,
    Unchanged,
}

impl<'a, T: UInteger, B: Behavior, const L: u128, const U: u128> std::ops::Deref
    for ClampGuard<'a, T, B, L, U>
{
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: UInteger, B: Behavior, const L: u128, const U: u128> std::ops::DerefMut
    for ClampGuard<'a, T, B, L, U>
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: UInteger, B: Behavior, const L: u128, const U: u128> AsRef<T>
    for ClampGuard<'a, T, B, L, U>
{
    #[inline(always)]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<'a, T: UInteger, B: Behavior, const L: u128, const U: u128> AsMut<T>
    for ClampGuard<'a, T, B, L, U>
{
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<'a, T: UInteger, B: Behavior, const L: u128, const U: u128> Drop
    for ClampGuard<'a, T, B, L, U>
{
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            if self.check() == GuardState::Changed {
                eprintln!("A `ClampGuard` was dropped without calling `commit` or `cancel` first");
            }
        }
    }
}

impl<'a, T: UInteger, B: Behavior, const L: u128, const U: u128> ClampGuard<'a, T, B, L, U> {
    pub(super) fn new(value: T, clamp: &'a mut T) -> Self {
        Self(value, clamp, std::marker::PhantomData)
    }

    #[inline(always)]
    pub fn check(&self) -> GuardState {
        if self.0 == *self.1 {
            GuardState::Unchanged
        } else {
            GuardState::Changed
        }
    }

    #[inline(always)]
    pub fn commit(self) -> Result<(), ClampError> {
        let mut me = std::mem::ManuallyDrop::new(self);
        let val = me.0;
        *me.1 = private::validate::<T, L, U>(val)?;
        Ok(())
    }

    #[inline(always)]
    pub fn cancel(self) {
        std::mem::forget(self);
    }
}

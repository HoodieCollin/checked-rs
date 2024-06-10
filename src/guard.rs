use crate::view::Validator;
use std::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
};

#[derive(Debug)]
pub struct Guard<'a, T: 'static, E, U: Validator<Item = T, Error = E>>(
    pub(crate) MaybeUninit<T>,
    pub(crate) &'a mut T,
    pub(crate) PhantomData<U>,
);

impl<'a, T, E, U: Validator<Item = T, Error = E>> std::ops::Deref for Guard<'a, T, E, U> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.assume_init_ref() }
    }
}

impl<'a, T, E, U: Validator<Item = T, Error = E>> std::ops::DerefMut for Guard<'a, T, E, U> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.assume_init_mut() }
    }
}

impl<'a, T, E, U: Validator<Item = T, Error = E>> AsRef<T> for Guard<'a, T, E, U> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        unsafe { self.0.assume_init_ref() }
    }
}

impl<'a, T, E, U: Validator<Item = T, Error = E>> AsMut<T> for Guard<'a, T, E, U> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.0.assume_init_mut() }
    }
}

impl<'a, T, E, U: Validator<Item = T, Error = E>> Drop for Guard<'a, T, E, U> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            eprintln!("A `Guard` was dropped without calling `commit` or `discard` first");
        }
    }
}

impl<'a, T, E, U: Validator<Item = T, Error = E>> Guard<'a, T, E, U> {
    #[inline(always)]
    pub(super) fn new(dst: &'a mut T) -> Self {
        Self(
            MaybeUninit::new(unsafe { std::ptr::read(&*dst) }),
            dst,
            PhantomData,
        )
    }

    #[inline(always)]
    pub fn is_changed(&self) -> bool
    where
        T: PartialEq,
    {
        let a = unsafe { self.0.assume_init_ref() };
        let b = &*self.1;

        a != b
    }

    #[inline(always)]
    pub fn check(&self) -> Result<(), E> {
        U::validate(unsafe { self.0.assume_init_ref() })
    }

    #[inline(always)]
    pub fn commit(self) -> Result<(), Self> {
        let mut this = std::mem::ManuallyDrop::new(self);

        match this.check() {
            Ok(_) => {
                *this.1 = unsafe { this.0.assume_init_read() };
                Ok(())
            }
            Err(_) => Err(ManuallyDrop::into_inner(this)),
        }
    }

    #[inline(always)]
    pub fn discard(self) {
        std::mem::forget(self);
    }
}

#[macro_export]
macro_rules! commit_or_bail {
    ($guard:expr) => {
        match $guard.check() {
            Ok(_) => {
                $guard.commit().unwrap();
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    };
}

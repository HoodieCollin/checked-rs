use anyhow::Result;

use crate::guard::Guard;

pub trait Validator: 'static + Copy {
    type Item;
    type Error;
    fn validate(item: &Self::Item) -> Result<(), Self::Error>;
}

#[derive(
    Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(transparent)]
pub struct View<T: 'static, E, U: Validator<Item = T, Error = E>>(T, std::marker::PhantomData<U>);

impl<T: std::fmt::Debug, E, U: Validator<Item = T, Error = E>> std::fmt::Debug for View<T, E, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("View").field(&self.0).finish()
    }
}

impl<T, E, U: Validator<Item = T, Error = E>> std::ops::Deref for View<T, E, U> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T, E, U: Validator<Item = T, Error = E>> AsRef<T> for View<T, E, U> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T, E, U: Validator<Item = T, Error = E>> View<T, E, U> {
    #[inline(always)]
    pub fn new(item: T) -> Self {
        Self(item, std::marker::PhantomData)
    }

    #[inline(always)]
    pub fn with_validator(item: T, _: U) -> Self {
        Self(item, std::marker::PhantomData)
    }

    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.0
    }

    #[inline(always)]
    #[must_use]
    pub fn modify<'a>(&'a mut self) -> Guard<'a, T, E, U> {
        Guard::new(&mut self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view() -> Result<()> {
        #[derive(Clone, Copy)]
        struct TestValidator;

        impl Validator for TestValidator {
            type Item = i32;
            type Error = anyhow::Error;

            fn validate(item: &Self::Item) -> Result<()> {
                if *item < 0 {
                    Err(anyhow::anyhow!("Value must be positive"))
                } else if *item % 2 == 0 && *item != 0 && *item <= 10 {
                    Err(anyhow::anyhow!(
                        "Value must be odd, or zero, or greater than 10"
                    ))
                } else if *item == 7 {
                    Err(anyhow::anyhow!("Value must not be 7"))
                } else {
                    Ok(())
                }
            }
        }

        let mut item = View::with_validator(0, TestValidator);
        let mut g = item.modify();

        *g = 1;
        assert_eq!(*g, 1);
        assert!(g.check().is_ok());

        *g = -1;
        assert_eq!(*g, -1);
        assert!(g.check().is_err());

        *g = 12;
        assert_eq!(*g, 12);
        assert!(g.check().is_ok());

        *g = 7;
        assert_eq!(*g, 7);

        let mut g = match g.commit() {
            Ok(_) => panic!("Expected error"),
            Err(g) => g,
        };

        // The guard's value should be unchanged if commit fails
        assert_eq!(*g, 7);

        *g = 100;
        assert!(g.commit().is_ok());

        // the guard is consumed by commit, so we can't check it again
        // the `View`'s value should be updated
        assert_eq!(&*item, &100);

        Ok(())
    }
}

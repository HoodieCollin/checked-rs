use anyhow::Result;

pub trait Validator: 'static + Copy {
    type Item;
    fn validate(item: &Self::Item) -> Result<()>;
}

#[derive(
    Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(transparent)]
pub struct View<T: 'static, U: Validator<Item = T>>(T, std::marker::PhantomData<U>);

impl<T: std::fmt::Debug, U: Validator<Item = T>> std::fmt::Debug for View<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("View").field(&self.0).finish()
    }
}

impl<T, U: Validator<Item = T>> std::ops::Deref for View<T, U> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T, U: Validator<Item = T>> std::ops::DerefMut for View<T, U> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T, U: Validator<Item = T>> AsRef<T> for View<T, U> {
    #[inline(always)]
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T, U: Validator<Item = T>> AsMut<T> for View<T, U> {
    #[inline(always)]
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T, U: Validator<Item = T>> Drop for View<T, U> {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        {
            eprintln!("Modified was dropped without calling `commit()` or `cancel()` first!")
        }
    }
}

impl<T, U: Validator<Item = T>> View<T, U> {
    #[inline(always)]
    pub fn new(item: T) -> Self {
        Self(item, std::marker::PhantomData)
    }

    #[inline(always)]
    pub fn with_validator(item: T, _: U) -> Self {
        Self(item, std::marker::PhantomData)
    }

    #[inline(always)]
    pub fn unwrap(self) -> T {
        let me = std::mem::ManuallyDrop::new(self);

        match U::validate(&me.0) {
            Ok(_) => unsafe { std::ptr::read(&me.0) },
            Err(e) => panic!("{:?}", e),
        }
    }

    #[inline(always)]
    pub fn try_unwrap(self) -> Result<T, Self> {
        let me = std::mem::ManuallyDrop::new(self);

        match U::validate(&me.0) {
            Ok(_) => Ok(unsafe { std::ptr::read(&me.0) }),
            Err(_) => Err(std::mem::ManuallyDrop::into_inner(me)),
        }
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        U::validate(&self.0).is_ok()
    }

    #[inline(always)]
    pub fn check(&self) -> Result<()> {
        U::validate(&self.0)
    }

    #[inline(always)]
    pub fn cancel(self) {
        let _ = std::mem::ManuallyDrop::new(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checked() -> Result<()> {
        #[derive(Copy, Clone)]
        struct CheckedIntValidator;

        impl Validator for CheckedIntValidator {
            type Item = i32;

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

        let mut item = View::with_validator(0, CheckedIntValidator);

        assert_eq!(*item, 0);
        assert!(item.is_valid());

        *item = 1;
        assert_eq!(*item, 1);
        assert!(item.check().is_ok());

        *item = -1;
        assert_eq!(*item, -1);
        assert!(item.check().is_err());

        *item = 12;
        assert_eq!(*item, 12);
        assert!(item.check().is_ok());

        *item = 7;
        assert_eq!(*item, 7);

        let item = match item.try_unwrap() {
            Ok(_) => panic!("Expected error"),
            Err(item) => {
                assert_eq!(*item, 7);
                item
            }
        };

        item.cancel();

        Ok(())
    }
}

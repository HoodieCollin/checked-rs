use crate::{view::Validator, InherentBehavior, UInteger, UIntegerLimits};
use anyhow::Result;

pub mod hard;
pub mod soft;

pub use self::{hard::HardClamp, soft::SoftClamp};

#[derive(Debug, Clone, Copy, Default)]
pub struct ClampValidator<T: UInteger, const L: u128, const U: u128>(std::marker::PhantomData<T>);

impl<T: UInteger, const L: u128, const U: u128> Validator for ClampValidator<T, L, U> {
    type Item = T;
    type Error = ClampError;

    fn validate(item: &Self::Item) -> Result<(), Self::Error> {
        crate::private::validate::<T, L, U>(*item)?;
        Ok(())
    }
}

pub trait EnumRepr<T: UInteger>:
    'static + Default + Eq + Ord + InherentBehavior + UIntegerLimits
{
    fn from_uint(value: T) -> Result<Self>;
    fn as_uint(&self) -> &T;

    fn into_uint(&self) -> T {
        *self.as_uint()
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum ClampError {
    #[error("Value too small: {val} (min: {min})")]
    TooSmall { val: u128, min: u128 },
    #[error("Value too large: {val} (max: {max})")]
    TooLarge { val: u128, max: u128 },
}

#[derive(Debug, Clone, Copy)]
pub enum Panicking {}

impl crate::Behavior for Panicking {
    fn add<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs + rhs;
        if (val) > max {
            panic!("Addition overflow");
        }
        if (val) < min {
            panic!("Addition underflow");
        }
        T::from_u128(val)
    }

    fn sub<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs - rhs;
        if (val) > max {
            panic!("Subtraction overflow");
        }
        if (val) < min {
            panic!("Subtraction underflow");
        }
        T::from_u128(val)
    }

    fn mul<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs * rhs;
        if (val) > max {
            panic!("Multiplication overflow");
        }
        if (val) < min {
            panic!("Multiplication underflow");
        }
        T::from_u128(val)
    }

    fn div<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs / rhs;
        if (val) > max {
            panic!("Division overflow");
        }
        if (val) < min {
            panic!("Division underflow");
        }
        T::from_u128(val)
    }

    fn rem<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs % rhs;
        if (val) > max {
            panic!("Remainder overflow");
        }
        if (val) < min {
            panic!("Remainder underflow");
        }
        T::from_u128(val)
    }

    fn bitand<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs & rhs;
        if (val) > max {
            panic!("Bitwise AND overflow");
        }
        if (val) < min {
            panic!("Bitwise AND underflow");
        }
        T::from_u128(val)
    }

    fn bitor<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs | rhs;
        if (val) > max {
            panic!("Bitwise OR overflow");
        }
        if (val) < min {
            panic!("Bitwise OR underflow");
        }
        T::from_u128(val)
    }

    fn bitxor<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs ^ rhs;
        if (val) > max {
            panic!("Bitwise XOR overflow");
        }
        if (val) < min {
            panic!("Bitwise XOR underflow");
        }
        T::from_u128(val)
    }

    fn shl<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs << rhs;
        if (val) > max {
            panic!("Bitwise shift left overflow");
        }
        if (val) < min {
            panic!("Bitwise shift left underflow");
        }
        T::from_u128(val)
    }

    fn shr<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs >> rhs;
        if (val) > max {
            panic!("Bitwise shift right overflow");
        }
        if (val) < min {
            panic!("Bitwise shift right underflow");
        }
        T::from_u128(val)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Saturating {}

impl crate::Behavior for Saturating {
    fn add<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs.saturating_add(rhs);
        T::from_u128(if (val) > max {
            max
        } else if (val) < min {
            min
        } else {
            val
        })
    }

    fn sub<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs.saturating_sub(rhs);
        T::from_u128(if (val) > max {
            max
        } else if (val) < min {
            min
        } else {
            val
        })
    }

    fn mul<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs.saturating_mul(rhs);
        T::from_u128(if (val) > max {
            max
        } else if (val) < min {
            min
        } else {
            val
        })
    }

    fn div<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs.saturating_div(rhs);
        T::from_u128(if (val) > max {
            max
        } else if (val) < min {
            min
        } else {
            val
        })
    }

    fn rem<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs % rhs;
        T::from_u128(if (val) > max {
            max
        } else if (val) < min {
            min
        } else {
            val
        })
    }

    fn bitand<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs & rhs;
        T::from_u128(if (val) > max {
            max
        } else if (val) < min {
            min
        } else {
            val
        })
    }

    fn bitor<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs | rhs;
        T::from_u128(if (val) > max {
            max
        } else if (val) < min {
            min
        } else {
            val
        })
    }

    fn bitxor<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs ^ rhs;
        T::from_u128(if (val) > max {
            max
        } else if (val) < min {
            min
        } else {
            val
        })
    }

    fn shl<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs << rhs;
        T::from_u128(if (val) > max {
            max
        } else if (val) < min {
            min
        } else {
            val
        })
    }

    fn shr<T: crate::UInteger>(lhs: T, rhs: T, min: u128, max: u128) -> T {
        let lhs = lhs.into_u128();
        let rhs = rhs.into_u128();

        let val = lhs >> rhs;
        T::from_u128(if (val) > max {
            max
        } else if (val) < min {
            min
        } else {
            val
        })
    }
}

#[cfg(test)]
mod tests {
    use checked_rs_macros::clamped;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_define() {
        #[clamped(u8; default = 1; behavior = Panicking)]
        #[derive(Debug, Clone, Copy)]
        pub enum Example {
            #[eq(0)]
            Nil,
            #[other]
            Valid,
            #[eq(u8::MAX)]
            Invalid,
        }

        let a: Example = Default::default();
        let b: Example = 254.into();
        let c = a + b;

        assert!(a.is_valid());
        assert!(b.is_valid());
        assert!(c.is_invalid());

        let d: Example = c - u8::MAX;

        assert!(d.is_nil());
    }
}

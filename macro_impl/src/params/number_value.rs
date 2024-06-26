use num_format::{Buffer, CustomFormat, Grouping};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_quote, spanned::Spanned};

use super::{NumberArg, NumberKind};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumberValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    USize(usize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    ISize(isize),
}

impl From<u8> for NumberValue {
    fn from(n: u8) -> Self {
        Self::U8(n)
    }
}

impl From<u16> for NumberValue {
    fn from(n: u16) -> Self {
        Self::U16(n)
    }
}

impl From<u32> for NumberValue {
    fn from(n: u32) -> Self {
        Self::U32(n)
    }
}

impl From<u64> for NumberValue {
    fn from(n: u64) -> Self {
        Self::U64(n)
    }
}

impl From<u128> for NumberValue {
    fn from(n: u128) -> Self {
        Self::U128(n)
    }
}

impl From<usize> for NumberValue {
    fn from(n: usize) -> Self {
        Self::USize(n)
    }
}

impl From<i8> for NumberValue {
    fn from(n: i8) -> Self {
        Self::I8(n)
    }
}

impl From<i16> for NumberValue {
    fn from(n: i16) -> Self {
        Self::I16(n)
    }
}

impl From<i32> for NumberValue {
    fn from(n: i32) -> Self {
        Self::I32(n)
    }
}

impl From<i64> for NumberValue {
    fn from(n: i64) -> Self {
        Self::I64(n)
    }
}

impl From<i128> for NumberValue {
    fn from(n: i128) -> Self {
        Self::I128(n)
    }
}

impl From<isize> for NumberValue {
    fn from(n: isize) -> Self {
        Self::ISize(n)
    }
}

impl std::ops::RangeBounds<NumberValue> for NumberValue {
    fn start_bound(&self) -> std::ops::Bound<&NumberValue> {
        std::ops::Bound::Included(self)
    }

    fn end_bound(&self) -> std::ops::Bound<&NumberValue> {
        std::ops::Bound::Excluded(self)
    }
}

impl std::fmt::Debug for NumberValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(n) => write!(f, "{}", n),
            Self::U16(n) => write!(f, "{}", n),
            Self::U32(n) => write!(f, "{}", n),
            Self::U64(n) => write!(f, "{}", n),
            Self::U128(n) => write!(f, "{}", n),
            Self::USize(n) => write!(f, "{}", n),
            Self::I8(n) => write!(f, "{}", n),
            Self::I16(n) => write!(f, "{}", n),
            Self::I32(n) => write!(f, "{}", n),
            Self::I64(n) => write!(f, "{}", n),
            Self::I128(n) => write!(f, "{}", n),
            Self::ISize(n) => write!(f, "{}", n),
        }
    }
}

impl From<NumberValue> for NumberKind {
    fn from(value: NumberValue) -> Self {
        value.kind()
    }
}

impl From<&NumberValue> for NumberKind {
    fn from(value: &NumberValue) -> Self {
        value.kind()
    }
}

impl std::fmt::Display for NumberValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U8(n) => write!(f, "{}", n),
            Self::U16(n) => write!(f, "{}", n),
            Self::U32(n) => write!(f, "{}", n),
            Self::U64(n) => write!(f, "{}", n),
            Self::U128(n) => write!(f, "{}", n),
            Self::USize(n) => write!(f, "{}", n),
            Self::I8(n) => write!(f, "{}", n),
            Self::I16(n) => write!(f, "{}", n),
            Self::I32(n) => write!(f, "{}", n),
            Self::I64(n) => write!(f, "{}", n),
            Self::I128(n) => write!(f, "{}", n),
            Self::ISize(n) => write!(f, "{}", n),
        }
    }
}

impl ToTokens for NumberValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::U8(n) => n.to_tokens(tokens),
            Self::U16(n) => n.to_tokens(tokens),
            Self::U32(n) => n.to_tokens(tokens),
            Self::U64(n) => n.to_tokens(tokens),
            Self::U128(n) => n.to_tokens(tokens),
            Self::USize(n) => n.to_tokens(tokens),
            Self::I8(n) => n.to_tokens(tokens),
            Self::I16(n) => n.to_tokens(tokens),
            Self::I32(n) => n.to_tokens(tokens),
            Self::I64(n) => n.to_tokens(tokens),
            Self::I128(n) => n.to_tokens(tokens),
            Self::ISize(n) => n.to_tokens(tokens),
        }
    }
}

impl NumberValue {
    pub fn kind(&self) -> NumberKind {
        match self {
            Self::U8(..) => NumberKind::U8,
            Self::U16(..) => NumberKind::U16,
            Self::U32(..) => NumberKind::U32,
            Self::U64(..) => NumberKind::U64,
            Self::U128(..) => NumberKind::U128,
            Self::USize(..) => NumberKind::USize,
            Self::I8(..) => NumberKind::I8,
            Self::I16(..) => NumberKind::I16,
            Self::I32(..) => NumberKind::I32,
            Self::I64(..) => NumberKind::I64,
            Self::I128(..) => NumberKind::I128,
            Self::ISize(..) => NumberKind::ISize,
        }
    }

    pub fn new(kind: NumberKind, n: i128) -> Self {
        match kind {
            NumberKind::U8 => Self::U8(n as u8),
            NumberKind::U16 => Self::U16(n as u16),
            NumberKind::U32 => Self::U32(n as u32),
            NumberKind::U64 => Self::U64(n as u64),
            NumberKind::U128 => Self::U128(n as u128),
            NumberKind::USize => Self::USize(n as usize),
            NumberKind::I8 => Self::I8(n as i8),
            NumberKind::I16 => Self::I16(n as i16),
            NumberKind::I32 => Self::I32(n as i32),
            NumberKind::I64 => Self::I64(n as i64),
            NumberKind::I128 => Self::I128(n as i128),
            NumberKind::ISize => Self::ISize(n as isize),
        }
    }

    pub fn into_usize(self) -> usize {
        match self {
            Self::U8(n) => n as usize,
            Self::U16(n) => n as usize,
            Self::U32(n) => n as usize,
            Self::U64(n) => n as usize,
            Self::U128(n) => n as usize,
            Self::USize(n) => n,
            Self::I8(n) => n as usize,
            Self::I16(n) => n as usize,
            Self::I32(n) => n as usize,
            Self::I64(n) => n as usize,
            Self::I128(n) => n as usize,
            Self::ISize(n) => n as usize,
        }
    }

    pub fn into_i128(self) -> i128 {
        match self {
            Self::U8(n) => n as i128,
            Self::U16(n) => n as i128,
            Self::U32(n) => n as i128,
            Self::U64(n) => n as i128,
            Self::U128(n) => n as i128,
            Self::USize(n) => n as i128,
            Self::I8(n) => n as i128,
            Self::I16(n) => n as i128,
            Self::I32(n) => n as i128,
            Self::I64(n) => n as i128,
            Self::I128(n) => n,
            Self::ISize(n) => n as i128,
        }
    }

    pub fn iter_to(self, end: Self) -> impl Iterator<Item = Self> {
        let kind = self.kind();
        let start = self.into_i128();
        let end = end.into_i128();

        (start..end).map(move |n| Self::new(kind, n))
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Self::U8(n) => *n == 0,
            Self::U16(n) => *n == 0,
            Self::U32(n) => *n == 0,
            Self::U64(n) => *n == 0,
            Self::U128(n) => *n == 0,
            Self::USize(n) => *n == 0,
            Self::I8(n) => *n == 0,
            Self::I16(n) => *n == 0,
            Self::I32(n) => *n == 0,
            Self::I64(n) => *n == 0,
            Self::I128(n) => *n == 0,
            Self::ISize(n) => *n == 0,
        }
    }

    pub fn is_positive(&self) -> bool {
        match self {
            Self::U8(n) => *n > 0,
            Self::U16(n) => *n > 0,
            Self::U32(n) => *n > 0,
            Self::U64(n) => *n > 0,
            Self::U128(n) => *n > 0,
            Self::USize(n) => *n > 0,
            Self::I8(n) => *n > 0,
            Self::I16(n) => *n > 0,
            Self::I32(n) => *n > 0,
            Self::I64(n) => *n > 0,
            Self::I128(n) => *n > 0,
            Self::ISize(n) => *n > 0,
        }
    }

    pub fn abs(&self) -> Self {
        match self {
            Self::U8(n) => Self::U8(*n),
            Self::U16(n) => Self::U16(*n),
            Self::U32(n) => Self::U32(*n),
            Self::U64(n) => Self::U64(*n),
            Self::U128(n) => Self::U128(*n),
            Self::USize(n) => Self::USize(*n),
            Self::I8(n) => Self::I8(n.abs()),
            Self::I16(n) => Self::I16(n.abs()),
            Self::I32(n) => Self::I32(n.abs()),
            Self::I64(n) => Self::I64(n.abs()),
            Self::I128(n) => Self::I128(n.abs()),
            Self::ISize(n) => Self::ISize(n.abs()),
        }
    }

    pub fn into_separated_string(&self) -> String {
        let format = CustomFormat::builder()
            .grouping(Grouping::Standard)
            .separator("_")
            .build()
            .expect("valid format");

        let mut buf = Buffer::new();

        match self {
            Self::U8(n) => buf.write_formatted(n, &format),
            Self::U16(n) => buf.write_formatted(n, &format),
            Self::U32(n) => buf.write_formatted(n, &format),
            Self::U64(n) => buf.write_formatted(n, &format),
            Self::U128(n) => buf.write_formatted(n, &format),
            Self::USize(n) => buf.write_formatted(n, &format),
            Self::I8(n) => buf.write_formatted(n, &format),
            Self::I16(n) => buf.write_formatted(n, &format),
            Self::I32(n) => buf.write_formatted(n, &format),
            Self::I64(n) => buf.write_formatted(n, &format),
            Self::I128(n) => buf.write_formatted(n, &format),
            Self::ISize(n) => buf.write_formatted(n, &format),
        };

        buf.to_string()
    }

    pub fn into_number_arg(&self) -> NumberArg {
        parse_quote!(#self)
    }

    pub fn add(self, rhs: Self) -> syn::Result<Self> {
        Ok(match (self, rhs) {
            (Self::U8(a), Self::U8(b)) => Self::U8(a + b),
            (Self::U16(a), Self::U16(b)) => Self::U16(a + b),
            (Self::U32(a), Self::U32(b)) => Self::U32(a + b),
            (Self::U64(a), Self::U64(b)) => Self::U64(a + b),
            (Self::U128(a), Self::U128(b)) => Self::U128(a + b),
            (Self::USize(a), Self::USize(b)) => Self::USize(a + b),
            (Self::I8(a), Self::I8(b)) => Self::I8(a + b),
            (Self::I16(a), Self::I16(b)) => Self::I16(a + b),
            (Self::I32(a), Self::I32(b)) => Self::I32(a + b),
            (Self::I64(a), Self::I64(b)) => Self::I64(a + b),
            (Self::I128(a), Self::I128(b)) => Self::I128(a + b),
            (Self::ISize(a), Self::ISize(b)) => Self::ISize(a + b),
            _ => {
                return Err(syn::Error::new(
                    self.span(),
                    format!("Invalid addition: {:?} + {:?}", self, rhs),
                ))
            }
        })
    }

    pub fn add_usize(self, rhs: usize) -> Self {
        match self {
            Self::U8(n) => Self::U8(n + rhs as u8),
            Self::U16(n) => Self::U16(n + rhs as u16),
            Self::U32(n) => Self::U32(n + rhs as u32),
            Self::U64(n) => Self::U64(n + rhs as u64),
            Self::U128(n) => Self::U128(n + rhs as u128),
            Self::USize(n) => Self::USize(n + rhs),
            Self::I8(n) => Self::I8(n + rhs as i8),
            Self::I16(n) => Self::I16(n + rhs as i16),
            Self::I32(n) => Self::I32(n + rhs as i32),
            Self::I64(n) => Self::I64(n + rhs as i64),
            Self::I128(n) => Self::I128(n + rhs as i128),
            Self::ISize(n) => Self::ISize(n + rhs as isize),
        }
    }

    pub fn sub(self, rhs: Self) -> syn::Result<Self> {
        Ok(match (self, rhs) {
            (Self::U8(a), Self::U8(b)) => Self::U8(a - b),
            (Self::U16(a), Self::U16(b)) => Self::U16(a - b),
            (Self::U32(a), Self::U32(b)) => Self::U32(a - b),
            (Self::U64(a), Self::U64(b)) => Self::U64(a - b),
            (Self::U128(a), Self::U128(b)) => Self::U128(a - b),
            (Self::USize(a), Self::USize(b)) => Self::USize(a - b),
            (Self::I8(a), Self::I8(b)) => Self::I8(a - b),
            (Self::I16(a), Self::I16(b)) => Self::I16(a - b),
            (Self::I32(a), Self::I32(b)) => Self::I32(a - b),
            (Self::I64(a), Self::I64(b)) => Self::I64(a - b),
            (Self::I128(a), Self::I128(b)) => Self::I128(a - b),
            (Self::ISize(a), Self::ISize(b)) => Self::ISize(a - b),
            _ => {
                return Err(syn::Error::new(
                    self.span(),
                    format!("Invalid subtraction: {:?} - {:?}", self, rhs),
                ))
            }
        })
    }

    pub fn sub_usize(self, rhs: usize) -> Self {
        match self {
            Self::U8(n) => Self::U8(n - rhs as u8),
            Self::U16(n) => Self::U16(n - rhs as u16),
            Self::U32(n) => Self::U32(n - rhs as u32),
            Self::U64(n) => Self::U64(n - rhs as u64),
            Self::U128(n) => Self::U128(n - rhs as u128),
            Self::USize(n) => Self::USize(n - rhs),
            Self::I8(n) => Self::I8(n - rhs as i8),
            Self::I16(n) => Self::I16(n - rhs as i16),
            Self::I32(n) => Self::I32(n - rhs as i32),
            Self::I64(n) => Self::I64(n - rhs as i64),
            Self::I128(n) => Self::I128(n - rhs as i128),
            Self::ISize(n) => Self::ISize(n - rhs as isize),
        }
    }
}

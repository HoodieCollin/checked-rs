use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parse;

/// Represents the size of number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberKind {
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,
}

impl Parse for NumberKind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        match ident.to_string().as_str() {
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "u128" => Ok(Self::U128),
            "usize" => Ok(Self::USize),
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "i128" => Ok(Self::I128),
            "isize" => Ok(Self::ISize),
            _ => Err(input.error("expected a number kind")),
        }
    }
}

impl ToTokens for NumberKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let kind = match self {
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
            Self::USize => "usize",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::I128 => "i128",
            Self::ISize => "isize",
        };

        tokens.extend(syn::parse_str::<TokenStream>(kind).unwrap());
    }
}

impl std::fmt::Display for NumberKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            Self::U8 => "u8",
            Self::U16 => "u16",
            Self::U32 => "u32",
            Self::U64 => "u64",
            Self::U128 => "u128",
            Self::USize => "usize",
            Self::I8 => "i8",
            Self::I16 => "i16",
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::I128 => "i128",
            Self::ISize => "isize",
        };

        write!(f, "{}", kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_parse, snapshot};

    #[test]
    fn parse_u8() {
        assert_parse!(NumberKind => { u8 } => { NumberKind::U8 });
    }

    #[test]
    fn parse_u16() {
        assert_parse!(NumberKind => { u16 } => { NumberKind::U16 });
    }

    #[test]
    fn parse_u32() {
        assert_parse!(NumberKind => { u32 } => { NumberKind::U32 });
    }

    #[test]
    fn parse_u64() {
        assert_parse!(NumberKind => { u64 } => { NumberKind::U64 });
    }

    #[test]
    fn parse_u128() {
        assert_parse!(NumberKind => { u128 } => { NumberKind::U128 });
    }

    #[test]
    fn parse_usize() {
        assert_parse!(NumberKind => { usize } => { NumberKind::USize });
    }

    #[test]
    fn parse_i8() {
        assert_parse!(NumberKind => { i8 } => { NumberKind::I8 });
    }

    #[test]
    fn parse_i16() {
        assert_parse!(NumberKind => { i16 } => { NumberKind::I16 });
    }

    #[test]
    fn parse_i32() {
        assert_parse!(NumberKind => { i32 } => { NumberKind::I32 });
    }

    #[test]
    fn parse_i64() {
        assert_parse!(NumberKind => { i64 } => { NumberKind::I64 });
    }

    #[test]
    fn parse_i128() {
        assert_parse!(NumberKind => { i128 } => { NumberKind::I128 });
    }

    #[test]
    fn parse_isize() {
        assert_parse!(NumberKind => { isize } => { NumberKind::ISize });
    }

    #[test]
    fn snapshot_u8() {
        snapshot!(NumberKind => { u8 });
    }

    #[test]
    fn snapshot_u16() {
        snapshot!(NumberKind => { u16 });
    }

    #[test]
    fn snapshot_u32() {
        snapshot!(NumberKind => { u32 });
    }

    #[test]
    fn snapshot_u64() {
        snapshot!(NumberKind => { u64 });
    }

    #[test]
    fn snapshot_u128() {
        snapshot!(NumberKind => { u128 });
    }

    #[test]
    fn snapshot_usize() {
        snapshot!(NumberKind => { usize });
    }

    #[test]
    fn snapshot_i8() {
        snapshot!(NumberKind => { i8 });
    }

    #[test]
    fn snapshot_i16() {
        snapshot!(NumberKind => { i16 });
    }

    #[test]
    fn snapshot_i32() {
        snapshot!(NumberKind => { i32 });
    }

    #[test]
    fn snapshot_i64() {
        snapshot!(NumberKind => { i64 });
    }

    #[test]
    fn snapshot_i128() {
        snapshot!(NumberKind => { i128 });
    }

    #[test]
    fn snapshot_isize() {
        snapshot!(NumberKind => { isize });
    }
}

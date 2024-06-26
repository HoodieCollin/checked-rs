use std::ops::{RangeFrom, RangeInclusive, RangeToInclusive};

use convert_case::{Case, Casing};
use num_format::{Buffer, CustomFormat, Grouping};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use rangemap::{RangeInclusiveSet, StepFns};
use rhai::{plugin::*, Engine};
use syn::{parenthesized, parse::Parse, parse_quote, spanned::Spanned};

/// Custom keywords used when parsing the `clamped` attribute.
pub mod kw {
    syn::custom_keyword!(derive);
    syn::custom_keyword!(default);
    syn::custom_keyword!(behavior);
    syn::custom_keyword!(lower);
    syn::custom_keyword!(upper);
    syn::custom_keyword!(min);
    syn::custom_keyword!(max);
    syn::custom_keyword!(Soft);
    syn::custom_keyword!(Hard);
    syn::custom_keyword!(Saturate);
    syn::custom_keyword!(Saturating);
    syn::custom_keyword!(Panic);
    syn::custom_keyword!(Panicking);
    syn::custom_keyword!(MIN);
    syn::custom_keyword!(MAX);
}

#[derive(Clone)]
pub struct DerivedTraits {
    pub derive_kw: kw::derive,
    pub paren: syn::token::Paren,
    pub traits: syn::punctuated::Punctuated<syn::TypePath, syn::Token![,]>,
}

impl Parse for DerivedTraits {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let derive_kw = input.parse()?;

        let content;
        parenthesized!(content in input);

        Ok(Self {
            derive_kw,
            paren: syn::token::Paren::default(),
            traits: content.parse_terminated(syn::TypePath::parse, syn::Token![,])?,
        })
    }
}

impl ToTokens for DerivedTraits {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let traits = &self.traits;
        tokens.extend(quote! {
            #[derive(#traits)]
        });
    }
}

#[derive(Clone)]
pub enum AsSoftOrHard {
    Soft {
        as_token: syn::Token![as],
        soft: kw::Soft,
    },
    Hard {
        as_token: syn::Token![as],
        hard: kw::Hard,
    },
}

impl Parse for AsSoftOrHard {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let as_token = input.parse()?;
        if input.peek(kw::Soft) {
            Ok(Self::Soft {
                as_token,
                soft: input.parse()?,
            })
        } else if input.peek(kw::Hard) {
            Ok(Self::Hard {
                as_token,
                hard: input.parse()?,
            })
        } else {
            Err(input.error("expected `Soft` or `Hard`"))
        }
    }
}

impl ToTokens for AsSoftOrHard {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Soft { as_token, soft } => {
                as_token.to_tokens(tokens);
                soft.to_tokens(tokens);
            }
            Self::Hard { as_token, hard } => {
                as_token.to_tokens(tokens);
                hard.to_tokens(tokens);
            }
        }
    }
}

/// Represents the `MIN` or `MAX` keyword.
#[derive(Clone)]
pub enum MinOrMax {
    Min(kw::MIN),
    Max(kw::MAX),
}

impl Parse for MinOrMax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::MIN) {
            Ok(Self::Min(input.parse()?))
        } else if input.peek(kw::MAX) {
            Ok(Self::Max(input.parse()?))
        } else {
            Err(input.error("expected `MIN` or `MAX`"))
        }
    }
}

impl ToTokens for MinOrMax {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Min(kw) => kw.to_tokens(tokens),
            Self::Max(kw) => kw.to_tokens(tokens),
        }
    }
}

/// Represents the `MIN` or `MAX` keyword.
#[derive(Clone)]
pub enum SemiOrComma {
    Semi(syn::Token![;]),
    Comma(syn::Token![,]),
}

impl Parse for SemiOrComma {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![;]) {
            Ok(Self::Semi(input.parse()?))
        } else if input.peek(syn::Token![,]) {
            Ok(Self::Comma(input.parse()?))
        } else {
            Err(input.error("expected `;` or `,`"))
        }
    }
}

impl ToTokens for SemiOrComma {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Semi(kw) => kw.to_tokens(tokens),
            Self::Comma(kw) => kw.to_tokens(tokens),
        }
    }
}

/// Represents the `Saturate` or `Saturating` keyword.
#[derive(Clone)]
pub enum SaturateOrSaturating {
    Saturate(kw::Saturate),
    Saturating(kw::Saturating),
}

impl Parse for SaturateOrSaturating {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::Saturate) {
            Ok(Self::Saturate(input.parse()?))
        } else if input.peek(kw::Saturating) {
            Ok(Self::Saturating(input.parse()?))
        } else {
            Err(input.error("expected `Saturate` or `Saturating`"))
        }
    }
}

impl ToTokens for SaturateOrSaturating {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Saturate(kw) => kw.to_tokens(tokens),
            Self::Saturating(kw) => kw.to_tokens(tokens),
        }
    }
}

/// Represents the `Saturate` or `Saturating` keyword.
#[derive(Clone)]
pub enum PanicOrPanicking {
    Panic(kw::Panic),
    Panicking(kw::Panicking),
}

impl Parse for PanicOrPanicking {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::Panic) {
            Ok(Self::Panic(input.parse()?))
        } else if input.peek(kw::Panicking) {
            Ok(Self::Panicking(input.parse()?))
        } else {
            Err(input.error("expected `Panic` or `Panicking`"))
        }
    }
}

impl ToTokens for PanicOrPanicking {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Panic(kw) => kw.to_tokens(tokens),
            Self::Panicking(kw) => kw.to_tokens(tokens),
        }
    }
}

#[derive(Clone)]
pub enum LowerOrMin {
    Lower(kw::lower),
    Min(kw::min),
}

impl Parse for LowerOrMin {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::lower) {
            Ok(Self::Lower(input.parse()?))
        } else if input.peek(kw::min) {
            Ok(Self::Min(input.parse()?))
        } else {
            Err(input.error("expected `lower` or `min`"))
        }
    }
}

impl ToTokens for LowerOrMin {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Lower(kw) => kw.to_tokens(tokens),
            Self::Min(kw) => kw.to_tokens(tokens),
        }
    }
}

#[derive(Clone)]
pub enum UpperOrMax {
    Upper(kw::upper),
    Max(kw::max),
}

impl Parse for UpperOrMax {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::upper) {
            Ok(Self::Upper(input.parse()?))
        } else if input.peek(kw::max) {
            Ok(Self::Max(input.parse()?))
        } else {
            Err(input.error("expected `upper` or `max`"))
        }
    }
}

impl ToTokens for UpperOrMax {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Upper(kw) => kw.to_tokens(tokens),
            Self::Max(kw) => kw.to_tokens(tokens),
        }
    }
}

/// Represents the behavior argument. It can be `Saturating` or `Panicking`.
#[derive(Clone)]
pub enum BehaviorArg {
    Saturating(SaturateOrSaturating),
    Panicking(PanicOrPanicking),
}

impl Parse for BehaviorArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::Saturate) || input.peek(kw::Saturating) {
            Ok(Self::Saturating(input.parse()?))
        } else if input.peek(kw::Panic) || input.peek(kw::Panicking) {
            Ok(Self::Panicking(input.parse()?))
        } else {
            Err(input.error("expected `Saturating` or `Panicking`"))
        }
    }
}

impl ToTokens for BehaviorArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::Saturating(..) => quote! {
                Saturating
            },
            Self::Panicking(..) => quote! {
                Panicking
            },
        });
    }
}

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

pub type NumberValueRangeSet = RangeInclusiveSet<NumberValue, NumberValueStepFns>;

pub struct NumberValueStepFns;

impl StepFns<NumberValue> for NumberValueStepFns {
    fn add_one(start: &NumberValue) -> NumberValue {
        start.add_usize(1)
    }

    fn sub_one(start: &NumberValue) -> NumberValue {
        start.sub_usize(1)
    }
}

#[derive(Clone)]
pub enum NumberValueRange {
    Full(NumberKind),
    From(RangeFrom<NumberValue>),
    To(RangeToInclusive<NumberValue>),
    Inclusive(RangeInclusive<NumberValue>),
}

impl NumberValueRange {
    fn check_matching_kinds(
        a: impl Into<NumberKind> + std::fmt::Debug + Clone,
        b: impl Into<NumberKind> + std::fmt::Debug + Clone,
    ) -> syn::Result<()> {
        let a_kind: NumberKind = a.clone().into();
        let b_kind: NumberKind = b.clone().into();

        if a_kind != b_kind {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("Number kinds do not match: {:?} != {:?}", a, b),
            ));
        }

        Ok(())
    }

    pub fn from_values(
        start: Option<NumberValue>,
        end: Option<NumberValue>,
        kind: NumberKind,
    ) -> syn::Result<Self> {
        Ok(match (start, end) {
            (Some(start), Some(end)) => {
                Self::check_matching_kinds(kind, &start)?;
                Self::check_matching_kinds(kind, &end)?;
                Self::Inclusive(start..=end)
            }
            (Some(start), None) => {
                Self::check_matching_kinds(kind, &start)?;
                Self::From(start..)
            }
            (None, Some(end)) => {
                Self::check_matching_kinds(kind, &end)?;
                Self::To(..=end)
            }
            (None, None) => Self::Full(kind),
        })
    }

    pub fn to_std_inclusive_range(
        &self,
        start_default: Option<NumberValue>,
        end_default: Option<NumberValue>,
    ) -> syn::Result<RangeInclusive<NumberValue>> {
        match self {
            Self::Full(kind) => {
                let start = start_default
                    .unwrap_or_else(|| NumberArg::new_min_constant(*kind).into_value(*kind));

                Self::check_matching_kinds(&start, *kind)?;

                let end = end_default
                    .unwrap_or_else(|| NumberArg::new_max_constant(*kind).into_value(*kind));

                Self::check_matching_kinds(&end, *kind)?;

                Ok(start..=end)
            }
            Self::From(range) => {
                let start = range.start.clone();
                let kind = start.kind();

                let end = end_default
                    .unwrap_or_else(|| NumberArg::new_max_constant(kind).into_value(kind));

                Self::check_matching_kinds(&end, kind)?;

                Ok(start..=end)
            }
            Self::To(range) => {
                let end = range.end.clone();
                let kind = end.kind();

                let start = start_default
                    .unwrap_or_else(|| NumberArg::new_min_constant(kind).into_value(kind));

                Self::check_matching_kinds(&start, kind)?;

                Ok(start..=end)
            }
            Self::Inclusive(range) => {
                let start = range.start();
                let end = range.end();

                Self::check_matching_kinds(start, end)?;

                Ok(*start..=*end)
            }
        }
    }
}

/// Represents the number argument. It can be a literal or a the MIN/MAX constant.
#[derive(Clone)]
pub enum NumberArg {
    Literal(syn::LitInt),
    ConstExpr {
        const_token: syn::Token![const],
        kind: NumberKind,
        block: syn::Block,
    },
    Constant {
        kind: NumberKind,
        dbl_colon: syn::Token![::],
        ident: MinOrMax,
    },
}

impl Parse for NumberArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::LitInt) {
            Ok(Self::Literal(input.parse()?))
        } else if input.peek(syn::Token![const]) {
            Ok(Self::ConstExpr {
                const_token: input.parse()?,
                kind: input.parse()?,
                block: input.parse()?,
            })
        } else {
            let kind = input.parse()?;
            let dbl_colon = input.parse()?;
            let ident: MinOrMax = input.parse()?;

            Ok(Self::Constant {
                kind,
                dbl_colon,
                ident,
            })
        }
    }
}

impl ToTokens for NumberArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Literal(lit) => lit.to_tokens(tokens),
            Self::ConstExpr { kind, .. } => tokens.extend(self.into_literal_as_tokens(*kind)),
            Self::Constant {
                kind,
                dbl_colon,
                ident,
            } => {
                let kind = kind.to_token_stream();
                let dbl_colon = dbl_colon.to_token_stream();
                let ident = ident.to_token_stream();

                tokens.extend(quote! {
                    #kind #dbl_colon #ident
                });
            }
        }
    }
}

impl std::fmt::Debug for NumberArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(lit) => write!(f, "{}", lit.to_token_stream().to_string()),
            Self::ConstExpr { kind, block, .. } => {
                write!(f, "const {} {}", kind, block.to_token_stream().to_string())
            }
            Self::Constant { kind, ident, .. } => {
                write!(f, "{}::{}", kind, ident.to_token_stream().to_string())
            }
        }
    }
}

impl std::fmt::Display for NumberArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl NumberArg {
    pub const LIMITS_INIT: (Option<Self>, Option<Self>) = (None, None);

    pub fn new_min_constant(kind: NumberKind) -> Self {
        Self::Constant {
            kind,
            dbl_colon: parse_quote!(::),
            ident: MinOrMax::Min(parse_quote!(MIN)),
        }
    }

    pub fn new_max_constant(kind: NumberKind) -> Self {
        Self::Constant {
            kind,
            dbl_colon: parse_quote!(::),
            ident: MinOrMax::Max(parse_quote!(MAX)),
        }
    }

    pub fn from_expr(expr: &syn::Expr) -> Self {
        parse_quote!(#expr)
    }

    pub fn from_lit(lit: &syn::LitInt) -> Self {
        Self::Literal(lit.clone())
    }

    pub fn from_range_expr(kind: NumberKind, expr: &syn::ExprRange) -> (Self, Self) {
        let start: Option<NumberArg> = expr.start.as_ref().map(|expr| parse_quote!(#expr));
        let end: Option<NumberArg> = expr.end.as_ref().map(|expr| parse_quote!(#expr));

        (
            start.unwrap_or_else(|| NumberArg::new_min_constant(kind)),
            end.unwrap_or_else(|| NumberArg::new_max_constant(kind)),
        )
    }

    pub fn min(&self, other: &Self, kind: NumberKind) -> Self {
        let a = self.into_value(kind);
        let b = other.into_value(kind);

        if a <= b {
            self.clone()
        } else {
            other.clone()
        }
    }

    pub fn max(&self, other: &Self, kind: NumberKind) -> Self {
        let a = self.into_value(kind);
        let b = other.into_value(kind);

        if a >= b {
            self.clone()
        } else {
            other.clone()
        }
    }

    pub fn into_value(&self, kind: NumberKind) -> NumberValue {
        match kind {
            NumberKind::U8 => NumberValue::U8(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::U16 => NumberValue::U16(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::U32 => NumberValue::U32(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::U64 => NumberValue::U64(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::U128 => NumberValue::U128(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::USize => NumberValue::USize(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::I8 => NumberValue::I8(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::I16 => NumberValue::I16(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::I32 => NumberValue::I32(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::I64 => NumberValue::I64(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::I128 => NumberValue::I128(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
            NumberKind::ISize => NumberValue::ISize(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => panic!("{}", e.to_string()),
            }),
        }
    }

    /// Output the value as a bare literal number in a token stream.
    pub fn into_literal_as_tokens(&self, kind: NumberKind) -> TokenStream {
        self.into_value(kind).into_token_stream()
    }

    /// Parse the value as a base 10 number.
    pub fn base10_parse<N>(&self) -> syn::Result<N>
    where
        N: std::str::FromStr,
        N::Err: std::fmt::Display,
    {
        match self {
            Self::Literal(lit) => lit.base10_parse::<N>(),
            Self::ConstExpr { kind, block, .. } => {
                match eval_const_expr(kind, block)?.to_string().parse() {
                    Ok(n) => Ok(n),
                    Err(e) => Err(syn::Error::new(block.span(), e)),
                }
            }
            Self::Constant {
                kind,
                dbl_colon: _,
                ident,
            } => {
                let n = match ident {
                    MinOrMax::Min(..) => match kind {
                        NumberKind::U8 => u8::MIN.to_string(),
                        NumberKind::U16 => u16::MIN.to_string(),
                        NumberKind::U32 => u32::MIN.to_string(),
                        NumberKind::U64 => u64::MIN.to_string(),
                        NumberKind::U128 => u128::MIN.to_string(),
                        NumberKind::USize => usize::MIN.to_string(),
                        NumberKind::I8 => i8::MIN.to_string(),
                        NumberKind::I16 => i16::MIN.to_string(),
                        NumberKind::I32 => i32::MIN.to_string(),
                        NumberKind::I64 => i64::MIN.to_string(),
                        NumberKind::I128 => i128::MIN.to_string(),
                        NumberKind::ISize => isize::MIN.to_string(),
                    },
                    MinOrMax::Max(..) => match kind {
                        NumberKind::U8 => u8::MAX.to_string(),
                        NumberKind::U16 => u16::MAX.to_string(),
                        NumberKind::U32 => u32::MAX.to_string(),
                        NumberKind::U64 => u64::MAX.to_string(),
                        NumberKind::U128 => u128::MAX.to_string(),
                        NumberKind::USize => usize::MAX.to_string(),
                        NumberKind::I8 => i8::MAX.to_string(),
                        NumberKind::I16 => i16::MAX.to_string(),
                        NumberKind::I32 => i32::MAX.to_string(),
                        NumberKind::I64 => i64::MAX.to_string(),
                        NumberKind::I128 => i128::MAX.to_string(),
                        NumberKind::ISize => isize::MAX.to_string(),
                    },
                };

                match str::parse(&n) {
                    Ok(n) => Ok(n),
                    Err(e) => Err(syn::Error::new(ident.span(), e)),
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct NumberRangeArg {
    pub start: Option<NumberArg>,
    pub dot_dot: Option<syn::Token![..]>,
    pub dot_dot_eq: Option<syn::Token![..=]>,
    pub end: Option<NumberArg>,
}

impl Parse for NumberRangeArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut start = None;
        let mut dot_dot = None;
        let mut dot_dot_eq = None;
        let mut end = None;

        let lookahead = input.lookahead1();

        if lookahead.peek(syn::Token![..=]) {
            dot_dot_eq = Some(input.parse()?);

            if !input.is_empty() {
                end = Some(input.parse()?);
            }
        } else if lookahead.peek(syn::Token![..]) {
            dot_dot = Some(input.parse()?);

            if !input.is_empty() {
                end = Some(input.parse()?);
            }
        } else if lookahead.peek(syn::LitInt) {
            start = Some(input.parse()?);

            if input.peek(syn::Token![..=]) {
                dot_dot_eq = Some(input.parse()?);

                if !input.is_empty() {
                    end = Some(input.parse()?);
                }
            } else if input.peek(syn::Token![..]) {
                dot_dot = Some(input.parse()?);

                if !input.is_empty() {
                    end = Some(input.parse()?);
                }
            }
        } else {
            return Err(lookahead.error());
        }

        Ok(Self {
            start,
            dot_dot,
            dot_dot_eq,
            end,
        })
    }
}

impl ToTokens for NumberRangeArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let start = self.start.as_ref();
        let dot_dot = self.dot_dot.as_ref();
        let dot_dot_eq = self.dot_dot_eq.as_ref();
        let end = self.end.as_ref();

        tokens.extend(quote! {
            #start #dot_dot #dot_dot_eq #end
        });
    }
}

impl std::fmt::Debug for NumberRangeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dot_dot = self.dot_dot.as_ref().map(|_| "..".to_string());
        let dot_dot_eq = self.dot_dot_eq.as_ref().map(|_| "..=".to_string());

        write!(
            f,
            "{}{}{}{}",
            self.start
                .as_ref()
                .map(|arg| arg.to_string())
                .unwrap_or_default(),
            dot_dot.unwrap_or_default(),
            dot_dot_eq.unwrap_or_default(),
            self.end
                .as_ref()
                .map(|arg| arg.to_string())
                .unwrap_or_default()
        )
    }
}

impl std::fmt::Display for NumberRangeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl NumberRangeArg {
    pub fn start_arg(&self, kind: NumberKind) -> NumberArg {
        self.start
            .as_ref()
            .cloned()
            .unwrap_or_else(|| NumberArg::new_min_constant(kind))
    }

    pub fn start_val(&self, kind: NumberKind) -> NumberValue {
        self.start_arg(kind).into_value(kind)
    }

    pub fn end_arg(&self, kind: NumberKind) -> NumberArg {
        self.end
            .as_ref()
            .cloned()
            .unwrap_or_else(|| NumberArg::new_max_constant(kind))
    }

    pub fn end_val(&self, kind: NumberKind) -> NumberValue {
        self.end_arg(kind).into_value(kind)
    }

    pub fn start_and_end_args(&self, kind: NumberKind) -> (NumberArg, NumberArg) {
        (self.start_arg(kind), self.end_arg(kind))
    }

    pub fn is_full_range(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    pub fn to_value_range(
        &self,
        kind: NumberKind,
        start_default: Option<NumberValue>,
        end_default: Option<NumberValue>,
    ) -> syn::Result<NumberValueRange> {
        NumberValueRange::from_values(
            self.start
                .as_ref()
                .map(|arg| arg.into_value(kind))
                .or(start_default),
            self.end
                .as_ref()
                .map(|arg| arg.into_value(kind))
                .or(end_default),
            kind,
        )
    }

    pub fn iter(&self, kind: NumberKind) -> impl Iterator<Item = NumberArg> {
        let start = self.start_val(kind);

        let end = {
            let val = self.end_val(kind);

            if self.dot_dot_eq.is_some() {
                val.add_usize(1)
            } else {
                val
            }
        };

        start.iter_to(end).map(|val| val.into_number_arg())
    }

    pub fn iter_values(&self, kind: NumberKind) -> impl Iterator<Item = NumberValue> {
        self.iter(kind).map(move |arg| arg.into_value(kind))
    }
}

#[derive(Clone)]
pub struct StrictNumberRangeArg(pub NumberRangeArg);

impl Parse for StrictNumberRangeArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let range: NumberRangeArg = input.parse()?;

        if range.start.is_none() && range.end.is_none() {
            Err(input.error("Should not be a full range"))
        } else {
            Ok(Self(range))
        }
    }
}

impl ToTokens for StrictNumberRangeArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

impl std::fmt::Debug for StrictNumberRangeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl std::fmt::Display for StrictNumberRangeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl StrictNumberRangeArg {
    pub fn start_arg(&self, kind: NumberKind) -> NumberArg {
        self.0.start_arg(kind)
    }

    pub fn start_val(&self, kind: NumberKind) -> NumberValue {
        self.start_arg(kind).into_value(kind)
    }

    pub fn end_arg(&self, kind: NumberKind) -> NumberArg {
        self.0.end_arg(kind)
    }

    pub fn end_val(&self, kind: NumberKind) -> NumberValue {
        self.end_arg(kind).into_value(kind)
    }

    pub fn start_and_end_args(&self, kind: NumberKind) -> (NumberArg, NumberArg) {
        (self.start_arg(kind), self.end_arg(kind))
    }

    pub fn to_value_range(
        &self,
        kind: NumberKind,
        start_default: Option<NumberValue>,
        end_default: Option<NumberValue>,
    ) -> syn::Result<NumberValueRange> {
        let start = self
            .0
            .start
            .as_ref()
            .map(|arg| arg.into_value(kind))
            .or(start_default);

        let end = self
            .0
            .end
            .as_ref()
            .map(|arg| arg.into_value(kind))
            .or(end_default);

        NumberValueRange::from_values(start, end, kind)
    }

    pub fn iter(&self, kind: NumberKind) -> impl Iterator<Item = NumberArg> {
        self.0.iter(kind)
    }
}

macro_rules! use_rhai_int {
    (
        declare {$($ty:ident),* $(,)?}
    ) => {
        paste::paste! {
            $(
                #[allow(dead_code)]
                #[export_module]
                mod [<rhai_ $ty>] {
                    #[allow(unused_imports)]
                    pub use std::$ty::*;
                }
            )*
        }
    };
    (
        register[$engine:ident] {$($ty:ident),* $(,)?}
    ) => {
        paste::paste! {
            $(
                let [< $ty _module >] = exported_module!([< rhai_ $ty >]);
                $engine.register_static_module(stringify!($ty), [< $ty _module >].into());
            )*
        }
    };
}

use_rhai_int! {
    declare {
        u8, u16, u32, u64, u128, usize,
        i8, i16, i32, i64, i128, isize,
    }
}

fn eval_const_expr(kind: &NumberKind, expr: &syn::Block) -> syn::Result<NumberValue> {
    let mut engine = Engine::new();

    use_rhai_int! {
        register[engine] {
            u8, u16, u32, u64, u128, usize,
            i8, i16, i32, i64, i128, isize,
        }
    }

    let stmts = &expr.stmts;

    if stmts.len() != 1 {
        return Err(syn::Error::new(expr.span(), "expected a single expression"));
    }

    let script = stmts[0].to_token_stream().to_string();

    Ok(match kind {
        NumberKind::U8 => match engine.eval_expression::<u8>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::U16 => match engine.eval_expression::<u16>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::U32 => match engine.eval_expression::<u32>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::U64 => match engine.eval_expression::<u64>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::U128 => match engine.eval_expression::<u128>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::USize => match engine.eval_expression::<usize>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::I8 => match engine.eval_expression::<i8>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::I16 => match engine.eval_expression::<i16>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::I32 => match engine.eval_expression::<i32>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::I64 => match engine.eval_expression::<i64>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::I128 => match engine.eval_expression::<i128>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
        NumberKind::ISize => match engine.eval_expression::<isize>(&script) {
            Ok(n) => n.into(),
            Err(err) => {
                return Err(syn::Error::new(
                    expr.span(),
                    format!("failed to evaluate expression: {}", err),
                ))
            }
        },
    })
}

pub struct Params {
    pub integer: NumberKind,
    pub derived_traits: Option<DerivedTraits>,
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub as_soft_or_hard: Option<AsSoftOrHard>,
    pub default_val: Option<NumberArg>,
    pub behavior_val: BehaviorArg,
    pub lower_limit: Option<NumberArg>,
    pub upper_limit: Option<NumberArg>,
    pub full_coverage: bool,
}

impl Params {
    pub fn mod_ident(&self) -> syn::Ident {
        format_ident!("clamped_{}", self.ident.to_string().to_case(Case::Snake))
    }

    pub fn guard_ident(&self) -> syn::Ident {
        format_ident!("{}Guard", &self.ident)
    }

    pub fn value_ident(&self) -> syn::Ident {
        format_ident!("{}Value", &self.ident)
    }

    pub fn other_ident(&self, other_name: &syn::Ident) -> syn::Ident {
        format_ident!("{}{}", other_name, self.value_ident())
    }

    /// Interpret the lower limit value as `NumberValue`.
    pub fn lower_limit_value(&self) -> Option<NumberValue> {
        self.lower_limit
            .as_ref()
            .map(|arg| arg.into_value(self.integer))
    }

    pub fn lower_limit_value_or_default(&self) -> NumberValue {
        self.lower_limit_value()
            .unwrap_or_else(|| NumberArg::new_min_constant(self.integer).into_value(self.integer))
    }

    /// Output the lower limit value as a bare literal in a token stream.
    pub fn lower_limit_token(&self) -> Option<TokenStream> {
        Some(syn::parse_str(&self.lower_limit_value().map(|val| val.to_string())?).unwrap())
    }

    pub fn lower_limit_token_or_default(&self) -> TokenStream {
        self.lower_limit_token()
            .unwrap_or_else(|| self.lower_limit_value_or_default().into_token_stream())
    }

    /// Interpret the upper limit value as `NumberValue`.
    pub fn upper_limit_value(&self) -> Option<NumberValue> {
        self.upper_limit
            .as_ref()
            .map(|arg| arg.into_value(self.integer))
    }

    pub fn upper_limit_value_or_default(&self) -> NumberValue {
        self.upper_limit_value()
            .unwrap_or_else(|| NumberArg::new_max_constant(self.integer).into_value(self.integer))
    }

    /// Output the upper limit value as a bare literal in a token stream.
    pub fn upper_limit_token(&self) -> Option<TokenStream> {
        Some(syn::parse_str(&self.upper_limit_value().map(|val| val.to_string())?).unwrap())
    }

    pub fn upper_limit_token_or_default(&self) -> TokenStream {
        self.upper_limit_token()
            .unwrap_or_else(|| self.upper_limit_value_or_default().into_token_stream())
    }

    /// Validate that an arbitrary value is within the lower and upper limit.
    pub fn check_if_out_of_bounds<T: Spanned + ToTokens>(
        &self,
        ast: &T,
        value: NumberValue,
    ) -> syn::Result<()> {
        let lower = self.lower_limit_value_or_default();
        let upper = self.upper_limit_value_or_default();

        if value < lower {
            return Err(syn::Error::new(
                ast.span(),
                format!(
                    "{:?} value: {} is less than lower limit: {}",
                    self.integer, value, lower
                ),
            ));
        }

        if value > upper {
            return Err(syn::Error::new(
                ast.span(),
                format!(
                    "{:?} value: {} is greater than upper limit: {}",
                    self.integer, value, upper
                ),
            ));
        }

        Ok(())
    }

    pub fn is_signed(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::I8
                | NumberKind::I16
                | NumberKind::I32
                | NumberKind::I64
                | NumberKind::I128
                | NumberKind::ISize
        )
    }

    /// Check if the number kind is `u16` or smaller.
    pub fn is_u16_or_smaller(&self) -> bool {
        matches!(self.integer, NumberKind::U8 | NumberKind::U16)
    }

    /// Check if the number kind is `u16` or larger.
    pub fn is_u16_or_larger(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::U16 | NumberKind::U32 | NumberKind::U64 | NumberKind::U128
        )
    }

    /// Check if the number kind is `u32` or smaller.
    pub fn is_u32_or_smaller(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::U8 | NumberKind::U16 | NumberKind::U32 | NumberKind::USize
        )
    }

    /// Check if the number kind is `u32` or larger.
    pub fn is_u32_or_larger(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::U32 | NumberKind::USize | NumberKind::U64 | NumberKind::U128
        )
    }

    /// Check if the number kind is `u64` or smaller.
    pub fn is_u64_or_smaller(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::U8
                | NumberKind::U16
                | NumberKind::U32
                | NumberKind::USize
                | NumberKind::U64
        )
    }

    /// Check if the number kind is `u64` or larger.
    pub fn is_u64_or_larger(&self) -> bool {
        matches!(self.integer, NumberKind::U64 | NumberKind::U128)
    }

    pub fn is_usize_or_smaller(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::U8 | NumberKind::U16 | NumberKind::U32 | NumberKind::USize
        )
    }

    pub fn is_usize_or_larger(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::USize | NumberKind::U64 | NumberKind::U128
        )
    }

    /// Check if the number kind is `u128` or smaller.
    pub fn is_u128_or_smaller(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::U8
                | NumberKind::U16
                | NumberKind::U32
                | NumberKind::U64
                | NumberKind::USize
                | NumberKind::U128
        )
    }

    pub fn is_i16_or_smaller(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::I8 | NumberKind::I16 | NumberKind::U8
        )
    }

    pub fn is_i16_or_larger(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::I16 | NumberKind::I32 | NumberKind::I64 | NumberKind::I128
        )
    }

    pub fn is_i32_or_smaller(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::I8
                | NumberKind::I16
                | NumberKind::I32
                | NumberKind::ISize
                | NumberKind::U8
                | NumberKind::U16
        )
    }

    pub fn is_i32_or_larger(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::I32 | NumberKind::I64 | NumberKind::ISize | NumberKind::I128
        )
    }

    pub fn is_i64_or_smaller(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::I8
                | NumberKind::I16
                | NumberKind::I32
                | NumberKind::ISize
                | NumberKind::I64
                | NumberKind::U8
                | NumberKind::U16
                | NumberKind::U32
        )
    }

    pub fn is_i64_or_larger(&self) -> bool {
        matches!(self.integer, NumberKind::I64 | NumberKind::I128)
    }

    pub fn is_isize_or_smaller(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::I8 | NumberKind::I16 | NumberKind::I32 | NumberKind::ISize | NumberKind::U8
        )
    }

    pub fn is_isize_or_larger(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::ISize | NumberKind::I64 | NumberKind::I128
        )
    }

    pub fn is_i128_or_smaller(&self) -> bool {
        matches!(
            self.integer,
            NumberKind::I8
                | NumberKind::I16
                | NumberKind::I32
                | NumberKind::I64
                | NumberKind::I128
                | NumberKind::U8
                | NumberKind::U16
                | NumberKind::U32
                | NumberKind::U64
                | NumberKind::USize
        )
    }
}

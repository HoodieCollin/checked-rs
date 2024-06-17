use std::iter::FusedIterator;

use proc_macro2::TokenStream;
use proc_macro_error::{abort, abort_call_site};
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse_quote, spanned::Spanned};

pub mod attr_params;
pub mod enum_variants;
pub mod struct_item;

/// Custom keywords used when parsing the `clamped` attribute.
mod kw {
    syn::custom_keyword!(default);
    syn::custom_keyword!(behavior);
    syn::custom_keyword!(lower);
    syn::custom_keyword!(upper);
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

/// Represents the size of number.
#[derive(Debug, Clone, Copy)]
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
            _ => abort!(ident, "expected a number type"),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl std::ops::Add for NumberValue {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
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
            _ => abort_call_site!("types must match"),
        }
    }
}

impl std::ops::Add<&Self> for NumberValue {
    type Output = Self;

    fn add(self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
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
            _ => abort_call_site!("types must match"),
        }
    }
}

impl std::ops::Add<usize> for NumberValue {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
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
}

impl std::ops::Sub for NumberValue {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
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
            _ => abort_call_site!("unsupported types"),
        }
    }
}

impl std::ops::Sub<&Self> for NumberValue {
    type Output = Self;

    fn sub(self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
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
            _ => abort_call_site!("unsupported types"),
        }
    }
}

impl std::ops::Sub<usize> for NumberValue {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
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

impl std::ops::RangeBounds<NumberValue> for NumberValue {
    fn start_bound(&self) -> std::ops::Bound<&NumberValue> {
        std::ops::Bound::Included(self)
    }

    fn end_bound(&self) -> std::ops::Bound<&NumberValue> {
        std::ops::Bound::Excluded(self)
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

    pub fn range(self, end: Self) -> NumberValueIter {
        NumberValueIter::new(self, end, 1.into())
    }
}

pub struct NumberValueIter {
    a: NumberValue,
    b: NumberValue,
    step: NumberValue,
}

impl Iterator for NumberValueIter {
    type Item = NumberValue;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.a + self.step;

        if next < self.b {
            self.a = next;
            Some(next)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for NumberValueIter {
    fn len(&self) -> usize {
        let diff = self.b - self.a;
        let step = self.step.into_usize();

        (diff.into_usize() + step - 1) / step
    }
}

impl DoubleEndedIterator for NumberValueIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let next = self.b - self.step;

        if next > self.a {
            self.b = next;
            Some(next)
        } else {
            None
        }
    }
}

impl FusedIterator for NumberValueIter {}

impl NumberValueIter {
    pub fn new(a: NumberValue, b: NumberValue, step: NumberValue) -> Self {
        match (a, b, step) {
            (NumberValue::U8(..), NumberValue::U8(..), NumberValue::U8(..)) => {}
            (NumberValue::U16(..), NumberValue::U16(..), NumberValue::U16(..)) => {}
            (NumberValue::U32(..), NumberValue::U32(..), NumberValue::U32(..)) => {}
            (NumberValue::U64(..), NumberValue::U64(..), NumberValue::U64(..)) => {}
            (NumberValue::U128(..), NumberValue::U128(..), NumberValue::U128(..)) => {}
            (NumberValue::USize(..), NumberValue::USize(..), NumberValue::USize(..)) => {}
            (NumberValue::I8(..), NumberValue::I8(..), NumberValue::I8(..)) => {}
            (NumberValue::I16(..), NumberValue::I16(..), NumberValue::I16(..)) => {}
            (NumberValue::I32(..), NumberValue::I32(..), NumberValue::I32(..)) => {}
            (NumberValue::I64(..), NumberValue::I64(..), NumberValue::I64(..)) => {}
            (NumberValue::I128(..), NumberValue::I128(..), NumberValue::I128(..)) => {}
            (NumberValue::ISize(..), NumberValue::ISize(..), NumberValue::ISize(..)) => {}
            _ => abort_call_site!("types must match"),
        }

        Self { a, b, step }
    }
}

/// Represents the number argument. It can be a literal or a the MIN/MAX constant.
#[derive(Clone)]
pub enum NumberArg {
    Literal(syn::LitInt),
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

impl NumberArg {
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

    pub fn into_value(&self, kind: NumberKind) -> NumberValue {
        match kind {
            NumberKind::U8 => NumberValue::U8(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::U16 => NumberValue::U16(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::U32 => NumberValue::U32(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::U64 => NumberValue::U64(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::U128 => NumberValue::U128(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::USize => NumberValue::USize(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::I8 => NumberValue::I8(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::I16 => NumberValue::I16(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::I32 => NumberValue::I32(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::I64 => NumberValue::I64(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::I128 => NumberValue::I128(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
            }),
            NumberKind::ISize => NumberValue::ISize(match self.base10_parse() {
                Ok(n) => n,
                Err(e) => abort_call_site!(e.to_string()),
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

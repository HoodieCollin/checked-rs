use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};
use syn::spanned::Spanned;

pub mod as_soft_or_hard;
pub mod behavior_arg;
pub mod derived_traits;
pub mod lower_or_min;
pub mod min_or_max;
pub mod number_arg;
pub mod number_arg_range;
pub mod number_kind;
pub mod number_value;
pub mod number_value_range;
pub mod panic_or_panicking;
pub mod saturate_or_saturating;
pub mod semi_or_colon;
pub mod upper_or_max;

pub use as_soft_or_hard::*;
pub use behavior_arg::*;
pub use derived_traits::*;
pub use lower_or_min::*;
pub use min_or_max::*;
pub use number_arg::*;
pub use number_arg_range::*;
pub use number_kind::*;
pub use number_value::*;
pub use number_value_range::*;
pub use panic_or_panicking::*;
pub use saturate_or_saturating::*;
pub use semi_or_colon::*;
pub use upper_or_max::*;

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

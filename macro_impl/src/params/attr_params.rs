use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::ToTokens;
use syn::{parse::Parse, parse_quote, spanned::Spanned};

use super::{kw, AsSoftOrHard, BehaviorArg, NumberArg, NumberKind, NumberValue, SemiOrComma};

/// Represents the parameters of the `clamped` attribute.
/// Only the `integer` and `default` parameters are required.
/// The `integer` parameter must be first while the order of the rest is not important.
#[derive(Clone)]
pub struct AttrParams {
    pub integer: syn::TypePath,
    pub as_soft_or_hard: Option<AsSoftOrHard>,
    pub integer_semi: Option<SemiOrComma>,
    pub default_kw: kw::default,
    pub default_eq: syn::Token![=],
    pub default_val: NumberArg,
    pub default_semi: Option<SemiOrComma>,
    pub behavior_kw: kw::behavior,
    pub behavior_eq: syn::Token![=],
    pub behavior_val: BehaviorArg,
    pub behavior_semi: Option<SemiOrComma>,
    pub lower_kw: Option<kw::lower>,
    pub lower_eq: Option<syn::Token![=]>,
    pub lower_val: Option<NumberArg>,
    pub lower_semi: Option<SemiOrComma>,
    pub upper_kw: Option<kw::upper>,
    pub upper_eq: Option<syn::Token![=]>,
    pub upper_val: Option<NumberArg>,
    pub upper_semi: Option<SemiOrComma>,
}

impl Parse for AttrParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let integer = input.parse()?;
        let mut as_soft_or_hard = None;
        let mut integer_semi = None;

        if input.peek(syn::Token![as]) {
            as_soft_or_hard = Some(input.parse()?);
        }

        if input.is_empty() {
            return Ok(Self {
                integer,
                as_soft_or_hard,
                integer_semi,
                default_kw: parse_quote!(default),
                default_eq: parse_quote!(=),
                default_val: parse_quote!(0),
                default_semi: None,
                behavior_kw: parse_quote!(behavior),
                behavior_eq: parse_quote!(=),
                behavior_val: parse_quote!(Panicking),
                behavior_semi: None,
                lower_kw: None,
                lower_eq: None,
                lower_val: None,
                lower_semi: None,
                upper_kw: None,
                upper_eq: None,
                upper_val: None,
                upper_semi: None,
            });
        } else {
            integer_semi = Some(input.parse::<SemiOrComma>()?);
        }

        let mut default_kw = None;
        let mut default_eq = None;
        let mut default_val = None;
        let mut default_semi = None;
        let mut behavior_kw = None;
        let mut behavior_eq = None;
        let mut behavior_val = None;
        let mut behavior_semi = None;
        let mut lower_kw = None;
        let mut lower_eq = None;
        let mut lower_val = None;
        let mut lower_semi = None;
        let mut upper_kw = None;
        let mut upper_eq = None;
        let mut upper_val = None;
        let mut upper_semi = None;

        let mut done = false;

        while !done {
            let mut found_semi = false;

            if input.peek(kw::default) {
                if default_kw.is_some() {
                    return Err(input.error("duplicate `default` param"));
                }

                default_kw = Some(input.parse::<kw::default>()?);
                default_eq = Some(input.parse::<syn::Token![=]>()?);
                default_val = Some(input.parse::<NumberArg>()?);
                if !input.is_empty() {
                    default_semi = Some(input.parse::<SemiOrComma>()?);
                    found_semi = true;
                }
            } else if input.peek(kw::behavior) {
                if behavior_kw.is_some() {
                    return Err(input.error("duplicate `behavior` param"));
                }

                behavior_kw = Some(input.parse::<kw::behavior>()?);
                behavior_eq = Some(input.parse::<syn::Token![=]>()?);
                behavior_val = Some(input.parse::<BehaviorArg>()?);
                if !input.is_empty() {
                    behavior_semi = Some(input.parse::<SemiOrComma>()?);
                    found_semi = true;
                }
            } else if input.peek(kw::lower) {
                if lower_kw.is_some() {
                    return Err(input.error("duplicate `lower` param"));
                }

                lower_kw = Some(input.parse::<kw::lower>()?);
                lower_eq = Some(input.parse::<syn::Token![=]>()?);
                lower_val = Some(input.parse::<NumberArg>()?);
                if !input.is_empty() {
                    lower_semi = Some(input.parse::<SemiOrComma>()?);
                    found_semi = true;
                }
            } else if input.peek(kw::upper) {
                if upper_kw.is_some() {
                    return Err(input.error("duplicate `upper` param"));
                }

                upper_kw = Some(input.parse::<kw::upper>()?);
                upper_eq = Some(input.parse::<syn::Token![=]>()?);
                upper_val = Some(input.parse::<NumberArg>()?);
                if !input.is_empty() {
                    upper_semi = Some(input.parse::<SemiOrComma>()?);
                    found_semi = true;
                }
            }

            if !found_semi {
                if default_kw.is_none() {
                    return Err(input.error("`default` param is required"));
                }

                done = true;
            }
        }

        if default_kw.is_none() {
            default_kw = Some(parse_quote!(default));
            default_eq = Some(parse_quote!(=));
            if let Some(lower_val) = &lower_val {
                default_val = Some(parse_quote!(#lower_val));
            } else {
                default_val = Some(parse_quote!(0));
            }
        }

        if behavior_kw.is_none() {
            behavior_kw = Some(parse_quote!(behavior));
            behavior_eq = Some(parse_quote!(=));
            behavior_val = Some(parse_quote!(Panicking));
        }

        let this = Self {
            integer,
            as_soft_or_hard,
            integer_semi,
            default_kw: default_kw.unwrap(),
            default_eq: default_eq.unwrap(),
            default_val: default_val.unwrap(),
            default_semi,
            behavior_kw: behavior_kw.unwrap(),
            behavior_eq: behavior_eq.unwrap(),
            behavior_val: behavior_val.unwrap(),
            behavior_semi,
            lower_kw,
            lower_eq,
            lower_val,
            lower_semi,
            upper_kw,
            upper_eq,
            upper_val,
            upper_semi,
        };

        if !this.is_u128_or_smaller() {
            abort!(this.integer, "expected number type")
        }

        match this.kind() {
            NumberKind::U8 => {
                if this.default_val.base10_parse::<u8>().is_err() {
                    abort!(this.default_val, "expected u8 value")
                }
            }
            NumberKind::U16 => {
                if this.default_val.base10_parse::<u16>().is_err() {
                    abort!(this.default_val, "expected u16 value")
                }
            }
            NumberKind::U32 => {
                if this.default_val.base10_parse::<u32>().is_err() {
                    abort!(this.default_val, "expected u32 value")
                }
            }
            NumberKind::U64 => {
                if this.default_val.base10_parse::<u64>().is_err() {
                    abort!(this.default_val, "expected u64 value")
                }
            }
            NumberKind::U128 => {
                if this.default_val.base10_parse::<u128>().is_err() {
                    abort!(this.default_val, "expected u128 value")
                }
            }
            NumberKind::USize => {
                if this.default_val.base10_parse::<usize>().is_err() {
                    abort!(this.default_val, "expected usize value")
                }
            }
            NumberKind::I8 => {
                if this.default_val.base10_parse::<i8>().is_err() {
                    abort!(this.default_val, "expected i8 value")
                }
            }
            NumberKind::I16 => {
                if this.default_val.base10_parse::<i16>().is_err() {
                    abort!(this.default_val, "expected i16 value")
                }
            }
            NumberKind::I32 => {
                if this.default_val.base10_parse::<i32>().is_err() {
                    abort!(this.default_val, "expected i32 value")
                }
            }
            NumberKind::I64 => {
                if this.default_val.base10_parse::<i64>().is_err() {
                    abort!(this.default_val, "expected i64 value")
                }
            }
            NumberKind::I128 => {
                if this.default_val.base10_parse::<i128>().is_err() {
                    abort!(this.default_val, "expected i128 value")
                }
            }
            NumberKind::ISize => {
                if this.default_val.base10_parse::<isize>().is_err() {
                    abort!(this.default_val, "expected isize value")
                }
            }
        }

        if this.default_value() < this.lower_limit_value() {
            abort!(
                this.default_val,
                "default value is less than lower bound value"
            )
        }

        if this.default_value() > this.upper_limit_value() {
            abort!(this.default_val, "default value exceeds upper bound value")
        }

        Ok(this)
    }
}

impl AttrParams {
    /// Get the number kind.
    pub fn kind(&self) -> NumberKind {
        self.integer
            .path
            .segments
            .iter()
            .last()
            .map(|s| match s.ident.to_string().as_str() {
                "u8" => NumberKind::U8,
                "u16" => NumberKind::U16,
                "u32" => NumberKind::U32,
                "u64" => NumberKind::U64,
                "u128" => NumberKind::U128,
                "usize" => NumberKind::USize,
                "i8" => NumberKind::I8,
                "i16" => NumberKind::I16,
                "i32" => NumberKind::I32,
                "i64" => NumberKind::I64,
                "i128" => NumberKind::I128,
                "isize" => NumberKind::ISize,
                _ => abort!(self.integer, "expected number type"),
            })
            .unwrap_or_else(|| abort!(self.integer, "expected number type"))
    }

    /// Interpret the default value as `NumberValue`.
    pub fn default_value(&self) -> NumberValue {
        self.default_val.into_value(self.kind())
    }

    /// Get the behavior type.
    pub fn behavior_type(&self) -> &BehaviorArg {
        &self.behavior_val
    }

    /// Interpret the lower limit value as `NumberValue`.
    pub fn lower_limit_value(&self) -> NumberValue {
        let kind = self.kind();
        if let Some(val) = &self.lower_val {
            val.into_value(kind)
        } else {
            NumberArg::new_min_constant(kind).into_value(kind)
        }
    }

    /// Output the lower limit value as a bare literal in a token stream.
    pub fn lower_limit_token(&self) -> TokenStream {
        syn::parse_str(&self.lower_limit_value().to_string()).unwrap()
    }

    /// Interpret the upper limit value as `NumberValue`.
    pub fn upper_limit_value(&self) -> NumberValue {
        let kind = self.kind();
        if let Some(val) = &self.upper_val {
            val.into_value(kind)
        } else {
            NumberArg::new_max_constant(kind).into_value(kind)
        }
    }

    /// Output the upper limit value as a bare literal in a token stream.
    pub fn upper_limit_token(&self) -> TokenStream {
        syn::parse_str(&self.upper_limit_value().to_string()).unwrap()
    }

    /// Validate that an arbitrary value is within the lower and upper limit.
    pub fn abort_if_out_of_bounds<T: Spanned + ToTokens>(&self, ast: &T, value: NumberValue) {
        if value < self.lower_limit_value() {
            abort!(
                ast,
                "{:?} value: {} is less than lower limit: {}",
                self.kind(),
                value,
                self.lower_limit_value()
            )
        }

        if value > self.upper_limit_value() {
            abort!(
                ast,
                "{:?} value: {} exceeds upper limit: {}",
                self.kind(),
                value,
                self.upper_limit_value()
            )
        }
    }

    pub fn is_signed(&self) -> bool {
        matches!(
            self.kind(),
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
        matches!(self.kind(), NumberKind::U8 | NumberKind::U16)
    }

    /// Check if the number kind is `u16` or larger.
    pub fn is_u16_or_larger(&self) -> bool {
        matches!(
            self.kind(),
            NumberKind::U16 | NumberKind::U32 | NumberKind::U64 | NumberKind::U128
        )
    }

    /// Check if the number kind is `u32` or smaller.
    pub fn is_u32_or_smaller(&self) -> bool {
        matches!(
            self.kind(),
            NumberKind::U8 | NumberKind::U16 | NumberKind::U32 | NumberKind::USize
        )
    }

    /// Check if the number kind is `u32` or larger.
    pub fn is_u32_or_larger(&self) -> bool {
        matches!(
            self.kind(),
            NumberKind::U32 | NumberKind::USize | NumberKind::U64 | NumberKind::U128
        )
    }

    /// Check if the number kind is `u64` or smaller.
    pub fn is_u64_or_smaller(&self) -> bool {
        matches!(
            self.kind(),
            NumberKind::U8
                | NumberKind::U16
                | NumberKind::U32
                | NumberKind::USize
                | NumberKind::U64
        )
    }

    /// Check if the number kind is `u64` or larger.
    pub fn is_u64_or_larger(&self) -> bool {
        matches!(self.kind(), NumberKind::U64 | NumberKind::U128)
    }

    pub fn is_usize_or_smaller(&self) -> bool {
        matches!(
            self.kind(),
            NumberKind::U8 | NumberKind::U16 | NumberKind::U32 | NumberKind::USize
        )
    }

    pub fn is_usize_or_larger(&self) -> bool {
        matches!(
            self.kind(),
            NumberKind::USize | NumberKind::U64 | NumberKind::U128
        )
    }

    /// Check if the number kind is `u128` or smaller.
    pub fn is_u128_or_smaller(&self) -> bool {
        matches!(
            self.kind(),
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
            self.kind(),
            NumberKind::I8 | NumberKind::I16 | NumberKind::U8
        )
    }

    pub fn is_i16_or_larger(&self) -> bool {
        matches!(
            self.kind(),
            NumberKind::I16 | NumberKind::I32 | NumberKind::I64 | NumberKind::I128
        )
    }

    pub fn is_i32_or_smaller(&self) -> bool {
        matches!(
            self.kind(),
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
            self.kind(),
            NumberKind::I32 | NumberKind::I64 | NumberKind::ISize | NumberKind::I128
        )
    }

    pub fn is_i64_or_smaller(&self) -> bool {
        matches!(
            self.kind(),
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
        matches!(self.kind(), NumberKind::I64 | NumberKind::I128)
    }

    pub fn is_isize_or_smaller(&self) -> bool {
        matches!(
            self.kind(),
            NumberKind::I8 | NumberKind::I16 | NumberKind::I32 | NumberKind::ISize | NumberKind::U8
        )
    }

    pub fn is_isize_or_larger(&self) -> bool {
        matches!(
            self.kind(),
            NumberKind::ISize | NumberKind::I64 | NumberKind::I128
        )
    }

    pub fn is_i128_or_smaller(&self) -> bool {
        matches!(
            self.kind(),
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

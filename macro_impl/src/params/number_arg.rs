use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use rhai::{plugin::*, Engine};
use syn::{parse::Parse, parse_quote, spanned::Spanned};

use super::{MinOrMax, NumberKind, NumberValue};

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

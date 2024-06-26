//! # checked-rs-macro-impl
//! > Implementation of the procedural macros for checked-rs. This crate is not intended to be used directly.
//! > Instead, you should use the `checked-rs` crate, which re-exports the public macros from this crate.
//!

use item::ClampedItem;
use params::NumberValueRange;
use proc_macro2::TokenStream;

pub mod common_impl;
pub mod enum_impl;
pub mod hard_impl;
pub mod soft_impl;

pub mod item;
pub mod params;

pub fn clamped(item: ClampedItem) -> TokenStream {
    let params = match item.params() {
        Ok(params) => params,
        Err(err) => return err.to_compile_error(),
    };

    match item {
        ClampedItem::Enum(item) => match enum_impl::define_mod(&params, &item.variants) {
            Ok(ts) => ts,
            Err(err) => err.to_compile_error(),
        },
        ClampedItem::Struct(item) => {
            let ranges = item
                .field
                .ranges
                .iter()
                .map(|range| {
                    NumberValueRange::from_values(
                        range
                            .start
                            .as_ref()
                            .map(|arg| arg.into_value(params.integer)),
                        range.end.as_ref().map(|arg| arg.into_value(params.integer)),
                        params.integer,
                    )
                })
                .collect::<syn::Result<Vec<_>>>();

            let ranges = match ranges {
                Ok(ranges) => ranges,
                Err(err) => return err.to_compile_error(),
            };

            match item.as_soft_or_hard {
                None => match soft_impl::define_mod(&params, &ranges) {
                    Ok(ts) => ts,
                    Err(err) => err.to_compile_error(),
                },
                Some(params::AsSoftOrHard::Soft { .. }) => {
                    match soft_impl::define_mod(&params, &ranges) {
                        Ok(ts) => ts,
                        Err(err) => err.to_compile_error(),
                    }
                }
                Some(params::AsSoftOrHard::Hard { .. }) => {
                    match hard_impl::define_mod(&params, &ranges) {
                        Ok(ts) => ts,
                        Err(err) => err.to_compile_error(),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::params::attr_params::AttrParams;

    use syn::parse_quote;

    #[test]
    fn test_enum_simple() {
        let generated = clamped(parse_quote! {
            #[usize]
            enum DoubleSentinel {
                Zero(0),
                Valid(..),
                Invalid(usize::MAX),
            }
        });

        println!(
            "{}",
            prettyplease::unparse(&syn::parse_file(&generated.to_string()).unwrap())
        );
    }

    #[test]
    fn test_enum_non_comprehensive() {
        let generated = clamped(parse_quote! {
            #[usize]
            enum DoubleSentinel {
                Ten(10),
                Twenty(20),
                Thirty(30),
            }
        });
    }

    #[test]
    fn test_enum_multiple_exacts() {
        let generated = clamped(parse_quote! {
            #[usize]
            enum SpecificValues {
                OneTwoOrSeven(1, 2, 7),
                AnythingElse(..),
            }
        });
    }

    #[test]
    fn test_enum_multiple_ranges() {
        let generated = clamped(parse_quote! {
            #[usize]
            enum HundredToThousand {
                Valid(..),
                Invalid(..100, 1000..)
            }
        });
    }

    #[test]
    fn test_enum_nested() {
        let generated = clamped(parse_quote! {
            #[usize]
            enum ResponseCode {
                Success[200..300] {
                    Okay(200),
                    Created(201),
                    Accepted(202),
                    Unknown(..),
                },
                Error {
                    Client[400..500] {
                        BadRequest(400),
                        Unauthorized(401),
                        PaymentRequired(402),
                        Forbidden(403),
                        NotFound(404),
                        Unknown(..)
                    },
                    Server[500..600] {
                        Internal(500),
                        NotImplemented(501),
                        BadGateway(502),
                        ServiceUnavailable(503),
                        GatewayTimeout(504),
                        Unknown(..)
                    }
                }
            }
        });
    }

    #[test]
    fn test_struct_soft() {
        let generated = clamped(parse_quote! {
            #[usize as Soft]
            struct TenOrLess(..=10);
        });
    }

    #[test]
    fn test_struct_hard() {
        let generated = clamped(parse_quote! {
            #[usize as Soft]
            struct TenOrMore(10..);
        });
    }

    #[test]
    fn test_struct_multiple_ranges() {
        let generated = clamped(parse_quote! {
            #[usize as Hard]
            struct LessThanTenOrBetween999And2000(..10, 1000..2000);
        });
    }
}

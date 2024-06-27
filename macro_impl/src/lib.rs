//! # checked-rs-macro-impl
//! > Implementation of the procedural macros for checked-rs. This crate is not intended to be used directly.
//! > Instead, you should use the `checked-rs` crate, which re-exports the public macros from this crate.
//!

use proc_macro2::TokenStream;

pub mod common_impl;
pub mod enum_impl;
pub mod hard_impl;
pub mod soft_impl;

pub mod item;
pub mod params;
mod range_seq;

use item::ClampedItem;
use params::NumberValueRange;

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
                .map(|range| NumberValueRange::from_arg_range(range.clone(), params.integer))
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

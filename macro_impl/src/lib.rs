//! # checked-rs-macro-impl
//!
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

#[cfg(test)]
macro_rules! snapshot {
    ($ty:ty => { $($tt:tt)* }) => {{
        let ts: $ty = match syn::parse2(quote::quote!($($tt)*)) {
            Ok(ts) => ts,
            Err(err) => panic!("Failed to parse as `{}`: {}", stringify!($ty), err),
        };

        insta::assert_snapshot!(&ts.to_token_stream().to_string());
    }};
    ($ty:ty => { $($tt:tt)* } => Formatted) => {{
        let ts: $ty = match syn::parse2(quote::quote!($($tt)*)) {
            Ok(ts) => ts,
            Err(err) => panic!("Failed to parse as `{}`: {}", stringify!($ty), err),
        };

        insta::assert_snapshot!(prettyplease::unparse(
            &syn::parse_file(&ts.to_token_stream().to_string()).unwrap()
        ));
    }};
}

#[cfg(test)]
pub(crate) use snapshot;

#[cfg(test)]
macro_rules! assert_parse {
    ($ty:ty => { $($tt:tt)* }) => {{
        match syn::parse2::<$ty>(quote::quote!($($tt)*)) {
            Ok(..) => { /* Success */ },
            Err(err) => panic!("Failed to parse: {}", err),
        };
    }};

    ($ty:ty => { $($tt:tt)* } => !) => {{
        match syn::parse2::<$ty>(quote::quote!($($tt)*)) {
            Ok(..) => panic!("Expected to fail parsing"),
            Err(..) => { /* Success */ }
        }
    }};

    ($ty:ty => { $($tt:tt)* } => { $($exp:tt)* }) => {{
        let x = match syn::parse2::<$ty>(quote::quote!($($tt)*)) {
            Ok(x) => x,
            Err(err) => panic!("Failed to parse: {}", err),
        };

        if !matches!(x, $($exp)*) {
            panic!("Expected {:?}, got {:?}", quote::quote!($($exp)*), quote::quote!(x));
        }
    }};
}

#[cfg(test)]
pub(crate) use assert_parse;

//! # checked-rs-macros
//!
//! > Procedural macros for the `checked-rs` crate. This crate should not be used directly.
//! > Instead, you should use the `checked-rs` crate, which re-exports the public macros from this crate.
//!
extern crate proc_macro;

use checked_rs_macro_impl::{clamped as clamped_impl, item::ClampedItem};
use syn::parse_macro_input;

#[proc_macro]
pub fn clamped(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as ClampedItem);

    proc_macro::TokenStream::from(clamped_impl(item))
}

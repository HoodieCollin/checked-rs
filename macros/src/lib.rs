extern crate proc_macro;

use checked_rs_macro_impl::{clamped, ops};
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

#[proc_macro_derive(CheckedRsOps, attributes(derive_deref_mut))]
#[proc_macro_error]
pub fn derive_ops(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    proc_macro::TokenStream::from(ops::derive_ops(input))
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn clamped(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as clamped::ClampParams);
    let item = parse_macro_input!(item as syn::Item);

    proc_macro::TokenStream::from(clamped::clamped(attr, item))
}

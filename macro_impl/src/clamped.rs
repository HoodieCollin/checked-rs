use proc_macro2::TokenStream;
use proc_macro_error::abort;

mod common_impl;
mod enum_impl;
mod hard_repr;
mod soft_repr;

use crate::params::{attr_params::AttrParams, AsSoftOrHard};

/// Generate the implementation for a clamped enum. This macro generates the following:
/// - An inner type that wraps the enum's value
/// - An implementation of `ClampedEnum` for the enum
/// - An implementation of `Deref` for the enum
/// - Implementations of various conversions for the enum
/// - Implementations of equality and comparison for the enum
/// - Implementations of various binary operations for the enum
pub fn clamped(attr: AttrParams, item: syn::Item) -> TokenStream {
    let is_enum = matches!(&item, syn::Item::Enum(_));

    if is_enum {
        enum_impl::define_mod(attr, item)
    } else {
        match attr.as_soft_or_hard {
            Some(AsSoftOrHard::Soft { .. }) => soft_repr::define_mod(attr, item),
            Some(AsSoftOrHard::Hard { .. }) => hard_repr::define_mod(attr, item),
            None => abort!(item, "The `clamped` attribute must specify either `as Soft` or `as Hard` when applied to a struct."),
        }
    }
}

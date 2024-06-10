//! # checked-rs-macro-impl
//! > Implementation of the procedural macros for checked-rs. This crate is not intended to be used directly.
//! > Instead, you should use the `checked-rs` crate, which re-exports the public macros from this crate.
//!
pub mod clamped;

#[doc(hidden)]
pub mod ops;

mod params;
mod variants;

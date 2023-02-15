/*!
Procedural macros for `pacdef`.
*/
#![warn(
    clippy::as_conversions,
    clippy::option_if_let_else,
    clippy::redundant_pub_crate,
    clippy::semicolon_if_nothing_returned,
    clippy::unused_self,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::use_self,
    clippy::wildcard_dependencies,
    missing_docs
)]

mod action;
mod register;

use proc_macro::TokenStream;

/// Derive (1) an iterator over the variants, (2) imports for the individual backends, and (3)
/// instatiation code for each backend.
#[proc_macro_derive(Register)]
pub fn register(input: TokenStream) -> TokenStream {
    register::register(input)
}

/// Derive public constants from each variant of the enum, such that the name of the constant is
/// the name of the variant in all caps, and the value is the name of the variant in lowercase.
#[proc_macro_derive(Action)]
pub fn action(input: TokenStream) -> TokenStream {
    action::action(input)
}

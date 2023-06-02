/*!
Procedural macros for `pacdef`.
*/

#![warn(
    clippy::as_conversions,
    clippy::cognitive_complexity,
    clippy::explicit_iter_loop,
    clippy::explicit_into_iter_loop,
    clippy::map_entry,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::option_if_let_else,
    clippy::redundant_pub_crate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnecessary_wraps,
    clippy::unused_self,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::use_self,
    clippy::wildcard_dependencies,
    missing_docs
)]

mod register;

use proc_macro::TokenStream;

/// Derive (1) an iterator over the variants, (2) imports for the individual backends, and (3)
/// instatiation code for each backend.
#[proc_macro_derive(Register)]
pub fn register(input: TokenStream) -> TokenStream {
    register::register(input)
}

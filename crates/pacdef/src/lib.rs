//! This library contains all logic that happens in `pacdef`.

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

pub(crate) mod backend;
#[allow(missing_docs)]
pub mod cli;

mod cmd;
mod config;
#[allow(clippy::unused_self, clippy::unnecessary_wraps)]
mod core;
mod env;
mod errors;
mod groups;
mod review;
mod search;
mod ui;

#[allow(unused_imports)]
mod prelude;

pub mod path;

pub use prelude::{Config, Error, Group};

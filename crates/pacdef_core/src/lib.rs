/*!
This library contains all logic that happens in `pacdef` under the hood.
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
    clippy::unused_self,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::use_self,
    clippy::wildcard_dependencies,
    missing_docs
)]

mod args;
mod backend;
mod cmd;
mod config;
mod core;
mod env;
mod errors;
mod grouping;
pub mod path;
mod review;
mod search;
mod ui;

pub use crate::args::get as get_args;
pub use crate::config::Config;
pub use crate::core::Pacdef;
pub use crate::errors::Error;
pub use crate::grouping::Group;
pub(crate) use crate::grouping::Package;

extern crate pacdef_macros;

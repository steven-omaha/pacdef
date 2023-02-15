/*!
This library contains all logic that happens in `pacdef` under the hood.
*/

#![warn(
    clippy::as_conversions,
    clippy::cognitive_complexity,
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

mod action;
mod args;
mod backend;
mod cmd;
mod config;
mod core;
mod env;
mod grouping;
mod path;
mod review;
mod search;
mod ui;

pub use crate::args::get as get_args;
pub use crate::config::Config;
pub use crate::core::Pacdef;
pub use crate::grouping::Group;
pub(crate) use crate::grouping::Package;
pub use crate::path::{get_config_path, get_group_dir};
pub use crate::search::NO_PACKAGES_FOUND;

extern crate pacdef_macros;

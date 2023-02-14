#![warn(
    clippy::as_conversions,
    clippy::option_if_let_else,
    clippy::redundant_pub_crate,
    clippy::unused_self,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::use_self,
    clippy::wildcard_dependencies
)]

mod action;
pub mod args;
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

pub use crate::config::Config;
pub use crate::core::Pacdef;
pub use crate::grouping::Group;
pub(crate) use crate::grouping::Package;
pub use crate::search::NO_PACKAGES_FOUND;

extern crate pacdef_macros;

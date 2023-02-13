#![warn(
    clippy::as_conversions,
    clippy::use_debug,
    clippy::unwrap_used,
    clippy::wildcard_dependencies,
    clippy::use_self,
    clippy::unused_self
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

extern crate macros;

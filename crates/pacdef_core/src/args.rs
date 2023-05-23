use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command, Parser, Subcommand, ValueEnum};
use path_absolutize::Absolutize;

use crate::action::*;
use crate::core::get_version_string;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    command: MainActions,
}

pub fn get() -> Args {
    Args::parse()
}

#[derive(Subcommand, Debug)]
enum MainActions {
    Group,
    Package,
    Version,
}

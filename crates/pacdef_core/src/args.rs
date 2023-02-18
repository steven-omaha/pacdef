use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use path_absolutize::Absolutize;

use crate::action::*;
use crate::core::get_version_string;

fn get_arg_parser() -> Command {
    let package_cmd = get_package_cmd();
    let group_cmd = get_group_cmd();
    let version_cmd = Command::new(VERSION).about("show version info");

    Command::new("pacdef")
        .about("declarative package manager for Linux")
        .version(get_version_string())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([package_cmd, group_cmd, version_cmd])
}

fn get_group_cmd() -> Command {
    let edit = Command::new(EDIT)
        .about("edit one or more existing group files")
        .arg_required_else_help(true)
        .arg(Arg::new("group").num_args(1..));

    let import = Command::new(IMPORT)
        .about("import one or more group files")
        .arg_required_else_help(true)
        .arg(Arg::new("files").num_args(1..));

    let list = Command::new(LIST).about("list names of imported groups");

    let new = Command::new(NEW)
        .about("create new group files")
        .arg_required_else_help(true)
        .arg(
            Arg::new("edit")
                .short('e')
                .long("edit")
                .help("edit the new group files after creation")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(Arg::new("groups").num_args(1..));

    let remove = Command::new(REMOVE)
        .about("remove one or more previously imported groups")
        .arg_required_else_help(true)
        .arg(Arg::new("groups").num_args(1..));

    let show = Command::new(SHOW)
        .about("show packages under an imported group")
        .arg_required_else_help(true)
        .arg(Arg::new("group").num_args(1..));

    Command::new("group")
        .arg_required_else_help(true)
        .about("TODO????")
        .subcommand_required(true)
        .subcommands([edit, import, list, new, remove, show])
}

fn get_package_cmd() -> Command {
    let sync = Command::new(SYNC).about("install packages from all imported groups");
    let clean = Command::new(CLEAN).about("remove unmanaged packages");
    let unmanaged =
        Command::new(UNMANAGED).about("show explicitly installed packages not managed by pacdef");
    let review = Command::new(REVIEW).about("review unmanaged packages");
    let search = Command::new(SEARCH)
        .about("search for packages which match a provided string literal or regex")
        .arg_required_else_help(true)
        .arg(Arg::new("string"));

    Command::new("package")
        .arg_required_else_help(true)
        .about("TODO????")
        .subcommand_required(true)
        .subcommands([clean, review, unmanaged, search, sync])
}

/// Get and parse the CLI arguments.
#[must_use]
pub fn get() -> clap::ArgMatches {
    get_arg_parser().get_matches()
}

pub fn get_absolutized_file_paths(arg_match: &ArgMatches) -> Result<Vec<PathBuf>> {
    Ok(arg_match
        .get_many::<String>("files")
        .context("getting files from args")?
        .map(PathBuf::from)
        .map(|path| {
            path.absolutize()
                .expect("absolute path should exist")
                .into_owned()
        })
        .collect())
}

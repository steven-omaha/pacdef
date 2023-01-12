use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command};
use path_absolutize::Absolutize;

use crate::action::*;

fn get_arg_parser() -> Command<'static> {
    let result = Command::new("pacdef")
        .about("declarative package manager for Arch Linux")
        .version("1.0.0-alpha")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new(CLEAN).about("remove unmanaged packages"))
        .subcommand(
            Command::new(EDIT)
                .about("edit one or more existing group files")
                .arg_required_else_help(true)
                .arg(Arg::new("group").multiple_values(true)),
        )
        .subcommand(Command::new(GROUPS).about("show names of imported groups"))
        .subcommand(
            Command::new(IMPORT)
                .about("import one or more group files")
                .arg_required_else_help(true)
                .arg(Arg::new("files").multiple_values(true)),
        )
        .subcommand(
            Command::new(NEW)
                .about("create new group files")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("edit")
                        .short('e')
                        .long("edit")
                        .help("edit the new group files after creation")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(Arg::new("groups").multiple_values(true)),
        )
        .subcommand(
            Command::new(REMOVE)
                .about("remove one or more previously imported groups")
                .arg_required_else_help(true)
                .arg(Arg::new("groups").multiple_values(true)),
        )
        .subcommand(
            Command::new(SEARCH)
                .about("search for packages which match a provided regex")
                .arg_required_else_help(true)
                .arg(Arg::new("string")),
        )
        .subcommand(
            Command::new(SHOW)
                .about("show packages under an imported group")
                .arg_required_else_help(true)
                .arg(Arg::new("group").multiple_values(true)),
        )
        .subcommand(Command::new(SYNC).about("install packages from all imported groups"))
        .subcommand(
            Command::new(UNMANAGED)
                .about("show explicitly installed packages not managed by pacdef"),
        )
        .subcommand(Command::new(VERSION).about("show version info"));
    result
}

#[must_use]
pub fn get() -> clap::ArgMatches {
    get_arg_parser().get_matches()
}

pub(crate) fn get_absolutized_file_paths(arg_match: &ArgMatches) -> Vec<PathBuf> {
    arg_match
        .get_many::<String>("files")
        .unwrap()
        .cloned()
        .map(PathBuf::from)
        .map(|path| path.absolutize().unwrap().into_owned())
        .collect()
}

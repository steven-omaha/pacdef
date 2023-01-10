use clap::{Arg, Command};

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

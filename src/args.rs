use clap::Arg;
use clap::Command;

pub const EDIT: &str = "edit";
pub const GROUPS: &str = "groups";
pub const SYNC: &str = "sync";
pub const UNMANAGED: &str = "unmanaged";
pub const VERSION: &str = "version";

fn get_arg_parser() -> Command<'static> {
    let result = Command::new("pacdef")
        .about("declarative package manager for Arch Linux")
        .version("1.0.0-alpha")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new(EDIT)
                .about("edit one or more existing group files")
                .arg_required_else_help(true)
                .arg(Arg::new("group")),
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

pub fn get_matched_arguments() -> clap::ArgMatches {
    get_arg_parser().get_matches()
}

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use path_absolutize::Absolutize;

use crate::action::*;
use crate::core::get_version_string;

const ACTION_NOT_MATCHED: &str = "could not match action";

/// Build the `pacdef` argument parser, with subcommands for `version`,
/// `group` and `package`.
fn build_cli() -> Command {
    let package_cmd = get_package_cmd();
    let group_cmd = get_group_cmd();
    let version_cmd = Command::new(VERSION).about("show version info");

    Command::new("pacdef")
        .about("multi-backend declarative package manager for Linux")
        .version(get_version_string())
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommands([group_cmd, package_cmd, version_cmd])
        .subcommand_value_name("subcommand")
        .disable_help_subcommand(true)
        .disable_version_flag(true)
}

/// Build the `pacdef group` subcommand.
fn get_group_cmd() -> Command {
    let edit = Command::new(EDIT)
        .about("edit one or more existing group")
        .arg_required_else_help(true)
        .arg(
            Arg::new("groups")
                .num_args(1..)
                .required(true)
                .help("a previously imported group"),
        )
        .visible_alias("e");

    let import = Command::new(IMPORT)
        .about("import one or more group files")
        .arg_required_else_help(true)
        .arg(
            Arg::new("files")
                .num_args(1..)
                .required(true)
                .help("the file to import as group"),
        )
        .visible_alias("i");

    let list = Command::new(LIST)
        .about("list names of imported groups")
        .visible_alias("l");

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
        .arg(Arg::new("groups").num_args(1..).required(true))
        .visible_alias("n");

    let remove = Command::new(REMOVE)
        .about("remove one or more previously imported groups")
        .arg_required_else_help(true)
        .arg(
            Arg::new("groups")
                .num_args(1..)
                .required(true)
                .help("a previously imported group that will be removed"),
        )
        .visible_alias("r");

    let show = Command::new(SHOW)
        .about("show packages under an imported group")
        .arg_required_else_help(true)
        .arg(
            Arg::new("groups")
                .num_args(1..)
                .required(true)
                .help("group file(s) to show"),
        )
        .visible_alias("s");

    Command::new("group")
        .arg_required_else_help(true)
        .about("manage groups")
        .visible_alias("g")
        .subcommand_required(true)
        .subcommands([edit, import, list, new, remove, show])
}

/// Build the `pacdef package` subcommand.
fn get_package_cmd() -> Command {
    let sync = Command::new(SYNC)
        .about("install packages from all imported groups")
        .visible_alias("sy")
        .arg(build_noconfirm_arg());

    let clean = Command::new(CLEAN)
        .about("remove unmanaged packages")
        .visible_alias("c")
        .arg(build_noconfirm_arg());

    let unmanaged = Command::new(UNMANAGED)
        .about("show explicitly installed packages not managed by pacdef")
        .visible_alias("u");

    let review = Command::new(REVIEW)
        .about("review unmanaged packages")
        .visible_alias("r");

    let search = Command::new(SEARCH)
        .visible_alias("se")
        .about("search for packages which match a provided regex")
        .arg_required_else_help(true)
        .arg(
            Arg::new("regex")
                .required(true)
                .help("the regular expression the package must match"),
        );

    Command::new("package")
        .arg_required_else_help(true)
        .about("manage packages")
        .visible_alias("p")
        .subcommand_required(true)
        .subcommands([clean, review, search, sync, unmanaged])
}

fn build_noconfirm_arg() -> Arg {
    Arg::new("noconfirm")
        .long("noconfirm")
        .help("do not ask for any confirmation")
        .action(ArgAction::SetTrue)
}

/// Get and parse the CLI arguments. Returns an instance of [`clap::ArgMatches`].
#[must_use]
pub fn get() -> Arguments {
    Arguments::from(build_cli().get_matches())
}

/// For each file argument, return the absolute path to the file.
pub fn get_absolutized_file_paths(file_names: &[String]) -> Result<Vec<PathBuf>> {
    Ok(file_names
        .iter()
        .map(PathBuf::from)
        .map(|path| {
            path.absolutize()
                .expect("absolute path should exist")
                .into_owned()
        })
        .collect())
}

#[derive(Debug)]
pub enum Arguments {
    Group(GroupArguments),
    Package(PackageArguments),
    Version,
}

#[derive(Debug, Clone)]
pub struct GroupArguments {
    pub(crate) action: GroupAction,
    pub(crate) edit: bool,
    pub(crate) groups: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PackageArguments {
    pub(crate) action: PackageAction,
    pub(crate) noconfirm: bool,
    pub(crate) regex: String,
}

impl GroupArguments {
    fn parse_args(args: &ArgMatches) -> Self {
        let action = GroupAction::try_from(args.subcommand_name().unwrap()).unwrap();
        let edit = args.get_one::<bool>("edit").cloned().unwrap_or_default();
        let groups = args
            .get_many::<String>("groups")
            .unwrap()
            .cloned()
            .collect();
        Self {
            action,
            edit,
            groups,
        }
    }
}

impl PackageArguments {
    fn parse_args(args: &ArgMatches) -> Self {
        dbg!(&args.subcommand_name().unwrap());
        let action = PackageAction::try_from(args.subcommand_name().unwrap()).unwrap();
        let noconfirm = args
            .try_get_one::<bool>("noconfirm")
            .map_or(false, |value| value.cloned().unwrap_or_default());
        let regex = args
            .try_get_one::<String>("regex")
            .map_or(String::default, |value| value.cloned().unwrap());

        Self {
            action,
            noconfirm,
            regex,
        }
    }
}

impl From<ArgMatches> for Arguments {
    fn from(value: ArgMatches) -> Self {
        let result = match value.subcommand() {
            Some(("group", args)) => Self::Group(GroupArguments::parse_args(args)),
            Some(("package", args)) => Self::Package(PackageArguments::parse_args(args)),
            Some(("version", _)) => Self::Version,
            _ => panic!("inconsistent argument definition"),
        };
        result
    }
}

use crate::core::get_version_string;
use clap::{Arg, ArgAction, Command};

#[must_use]
pub(super) fn build_cli() -> Command {
    let package_cmd = get_package_cmd();
    let group_cmd = get_group_cmd();
    let version_cmd = Command::new("version").about("show version info");

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
    let edit = Command::new("edit")
        .about("edit one or more existing group")
        .arg_required_else_help(true)
        .arg(
            Arg::new("groups")
                .num_args(1..)
                .required(true)
                .help("a previously imported group"),
        )
        .visible_alias("ed");

    let export = Command::new("export")
        .about("export one or more group files")
        .arg_required_else_help(true)
        .arg(
            Arg::new("groups")
                .num_args(1..)
                .required(true)
                .help("the file to export as group"),
        )
        .visible_alias("ex");

    let import = Command::new("import")
        .about("import one or more group files")
        .arg_required_else_help(true)
        .arg(
            Arg::new("groups")
                .num_args(1..)
                .required(true)
                .help("the file to import as group"),
        )
        .visible_alias("i");

    let list = Command::new("list")
        .about("list names of imported groups")
        .visible_alias("l");

    let new = Command::new("new")
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

    let remove = Command::new("remove")
        .about("remove one or more previously imported groups")
        .arg_required_else_help(true)
        .arg(
            Arg::new("groups")
                .num_args(1..)
                .required(true)
                .help("a previously imported group that will be removed"),
        )
        .visible_alias("r");

    let show = Command::new("show")
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
        .subcommands([edit, export, import, list, new, remove, show])
}

/// Build the `pacdef package` subcommand.
fn get_package_cmd() -> Command {
    let sync = Command::new("sync")
        .about("install packages from all imported groups")
        .visible_alias("sy")
        .arg(build_noconfirm_arg());

    let clean = Command::new("clean")
        .about("remove unmanaged packages")
        .visible_alias("c")
        .arg(build_noconfirm_arg());

    let unmanaged = Command::new("unmanaged")
        .about("show explicitly installed packages not managed by pacdef")
        .visible_alias("u");

    let review = Command::new("review")
        .about("review unmanaged packages")
        .visible_alias("r");

    let search = Command::new("search")
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

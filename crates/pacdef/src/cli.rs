//! The clap declarative command line interface

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    version,
    author,
    arg_required_else_help(true),
    subcommand_required(true),
    disable_help_subcommand(true),
    disable_version_flag(true)
)]
/// multi-backend declarative package manager for Linux
pub struct MainArguments {
    #[command(subcommand)]
    pub subcommand: MainSubcommand,
}

#[derive(Subcommand)]
pub enum MainSubcommand {
    Group(GroupArguments),
    Package(PackageArguments),
    Version(VersionArguments),
}

#[derive(Args)]
#[command(
    arg_required_else_help(true),
    visible_alias("g"),
    subcommand_required(true)
)]
/// manage groups
pub struct GroupArguments {
    #[command(subcommand)]
    pub group_action: GroupAction,
}

#[derive(Subcommand)]
pub enum GroupAction {
    Edit(EditGroupAction),
    Export(ExportGroupAction),
    Import(ImportGroupAction),
    List(ListGroupAction),
    New(NewGroupAction),
    Remove(RemoveGroupAction),
    Show(ShowGroupAction),
}

#[derive(Args)]
#[command(arg_required_else_help(true), visible_alias("ed"))]
/// edit one or more existing group
pub struct EditGroupAction {
    #[arg(required(true), num_args(1..))]
    /// a previously imported group
    pub edit_groups: Vec<String>,
}

#[derive(Args)]
#[command(arg_required_else_help(true), visible_alias("ex"))]
/// export one or more group files
pub struct ExportGroupAction {
    #[arg(required(true), num_args(1..))]
    /// the file to export as group
    pub export_groups: Vec<String>,

    #[arg(short, long)]
    /// (optional) the directory under which to save the group
    pub output_dir: Option<PathBuf>,

    #[arg(short, long)]
    /// overwrite output files if they exist
    pub force: bool,
}

#[derive(Args)]
#[command(arg_required_else_help(true), visible_alias("i"))]
/// import one or more group files
pub struct ImportGroupAction {
    #[arg(required(true), num_args(1..))]
    /// the file to import as group
    pub import_groups: Vec<String>,
}

#[derive(Args)]
#[command(visible_alias("l"))]
/// list names of imported groups
pub struct ListGroupAction {}

#[derive(Args)]
#[command(arg_required_else_help(true), visible_alias("n"))]
/// create new group files
pub struct NewGroupAction {
    #[arg(required(true), num_args(1..))]
    /// the groups to create
    pub new_groups: Vec<String>,

    #[arg(short, long)]
    /// edit the new group files after creation
    pub edit: bool,
}

#[derive(Args)]
#[command(arg_required_else_help(true), visible_alias("r"))]
/// remove one or more previously imported groups
pub struct RemoveGroupAction {
    #[arg(required(true), num_args(1..))]
    /// a previously imported group that will be removed
    pub remove_groups: Vec<String>,
}

#[derive(Args)]
#[command(arg_required_else_help(true), visible_alias("s"))]
/// show packages under an imported group
pub struct ShowGroupAction {
    #[arg(required(true), num_args(1..))]
    /// group file(s) to show
    pub show_groups: Vec<String>,
}

#[derive(Args)]
#[command(
    arg_required_else_help(true),
    subcommand_required(true),
    visible_alias("p")
)]
/// manage packages
pub struct PackageArguments {
    #[command(subcommand)]
    pub package_action: PackageAction,
}

#[derive(Subcommand)]
pub enum PackageAction {
    Clean(CleanPackageAction),
    Review(ReviewPackageAction),
    Search(SearchPackageAction),
    Sync(SyncPackageAction),
    Unmanaged(UnmanagedPackageAction),
}

#[derive(Args)]
#[command(visible_alias("c"))]
/// remove unmanaged packages
pub struct CleanPackageAction {
    #[arg(long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
#[command(visible_alias("r"))]
/// review unmanaged packages
pub struct ReviewPackageAction {}

#[derive(Args)]
#[command(arg_required_else_help(true), visible_alias("se"))]
/// search for packages which match a provided regex
pub struct SearchPackageAction {
    #[arg(required(true))]
    /// the regular expression the package must match
    pub regex: String,
}

#[derive(Args)]
#[command(visible_alias("sy"))]
/// install packages from all imported groups
pub struct SyncPackageAction {
    #[arg(long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
#[command(visible_alias("u"))]
/// show explicitly installed packages not managed by pacdef
pub struct UnmanagedPackageAction {}

#[derive(Args)]
pub struct VersionArguments {}

use super::datastructure::{
    Arguments, Edit, Force, GroupAction, Groups, Noconfirm, OutputDir, PackageAction, Regex,
};

const ARGS_CONSISTENT: &str = "argument declaration and parsing must be consistent";

pub(super) fn parse(args: clap::ArgMatches) -> Arguments {
    match args.subcommand() {
        Some(("group", args)) => Arguments::Group(parse_group_args(args)),
        Some(("package", args)) => Arguments::Package(parse_package_args(args)),
        Some(("version", _)) => Arguments::Version,
        Some(value) => panic!("main subcommand was not matched: {value:?}"),
        None => unreachable!("prevented by clap"),
    }
}

fn parse_group_args(args: &clap::ArgMatches) -> GroupAction {
    match args.subcommand() {
        Some(("edit", args)) => GroupAction::Edit(get_groups(args)),
        Some(("export", args)) => {
            GroupAction::Export(get_groups(args), get_output_dir(args), get_force(args))
        }
        Some(("import", args)) => GroupAction::Import(get_groups(args)),
        Some(("list", _)) => GroupAction::List,
        Some(("new", args)) => GroupAction::New(get_groups(args), get_edit(args)),
        Some(("remove", args)) => GroupAction::Remove(get_groups(args)),
        Some(("show", args)) => GroupAction::Show(get_groups(args)),
        Some(value) => panic!("group subcommand was not matched: {value:?}"),
        None => unreachable!("prevented by clap"),
    }
}

fn parse_package_args(args: &clap::ArgMatches) -> PackageAction {
    match args.subcommand() {
        Some(("clean", args)) => PackageAction::Clean(get_noconfirm(args)),
        Some(("review", _)) => PackageAction::Review,
        Some(("search", args)) => PackageAction::Search(get_regex(args)),
        Some(("sync", args)) => PackageAction::Sync(get_noconfirm(args)),
        Some(("unmanaged", _)) => PackageAction::Unmanaged,
        Some(value) => panic!("package subcommand was not matched: {value:?}"),
        None => unreachable!("prevented by clap"),
    }
}

fn get_one_arg<T>(args: &clap::ArgMatches, id: &str) -> T
where
    T: std::any::Any + Clone + Sync + Send + 'static,
{
    args.get_one::<T>(id).expect(ARGS_CONSISTENT).to_owned()
}

fn get_regex(args: &clap::ArgMatches) -> Regex {
    Regex(get_one_arg(args, "regex"))
}

fn get_noconfirm(args: &clap::ArgMatches) -> Noconfirm {
    Noconfirm(get_one_arg(args, "noconfirm"))
}

fn get_edit(args: &clap::ArgMatches) -> Edit {
    Edit(get_one_arg(args, "edit"))
}

fn get_groups(args: &clap::ArgMatches) -> Groups {
    Groups(
        args.get_many::<String>("groups")
            .expect(ARGS_CONSISTENT)
            .cloned()
            .collect(),
    )
}

fn get_output_dir(args: &clap::ArgMatches) -> OutputDir {
    OutputDir(args.get_one::<String>("output_dir").cloned())
}

fn get_force(args: &clap::ArgMatches) -> Force {
    Force(get_one_arg(args, "force"))
}

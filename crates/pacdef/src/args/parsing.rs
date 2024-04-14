use super::datastructure::*;

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
    use GroupAction::*;

    match args.subcommand() {
        Some(("edit", args)) => Edit(get_groups(args)),
        Some(("export", args)) => Export(get_groups(args), get_output_dir(args), get_force(args)),
        Some(("import", args)) => Import(get_groups(args)),
        Some(("list", _)) => List,
        Some(("new", args)) => New(get_groups(args), get_edit(args)),
        Some(("remove", args)) => Remove(get_groups(args)),
        Some(("show", args)) => Show(get_groups(args)),
        Some(value) => panic!("group subcommand was not matched: {value:?}"),
        None => unreachable!("prevented by clap"),
    }
}

fn parse_package_args(args: &clap::ArgMatches) -> PackageAction {
    use PackageAction::*;

    match args.subcommand() {
        Some(("clean", args)) => Clean(get_noconfirm(args)),
        Some(("review", _)) => Review,
        Some(("search", args)) => Search(get_regex(args)),
        Some(("sync", args)) => Sync(get_noconfirm(args)),
        Some(("unmanaged", _)) => Unmanaged,
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

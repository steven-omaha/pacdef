use super::cli::build_cli;
use super::datastructure::*;
use super::parsing::parse;

use rstest::rstest;

#[rstest]
#[case(vec!["pacdef", "group", "list"],
       Arguments::Group(GroupAction::List))]
#[case(vec!["pacdef", "g", "l"],
       Arguments::Group(GroupAction::List))]
#[case(vec!["pacdef", "version"],
       Arguments::Version)]
#[case(vec!["pacdef", "group", "export", "-o", "/tmp", "--force", "base"],
       Arguments::Group(GroupAction::Export(Groups(vec!["base".to_string()]), OutputDir(Some("/tmp".to_string())), Force(true))))]
#[case(vec!["pacdef", "package", "sync"],
       Arguments::Package(PackageAction::Sync(Noconfirm(false))))]
#[case(vec!["pacdef", "package", "sync", "--noconfirm"],
       Arguments::Package(PackageAction::Sync(Noconfirm(true))))]
#[case(vec!["pacdef", "p", "se", "myregex"],
       Arguments::Package(PackageAction::Search(Regex("myregex".to_string()))))]
fn arg_parsing(#[case] input: Vec<&str>, #[case] expected: Arguments) {
    let args = build_cli().get_matches_from(input);
    let parsed = parse(args);

    assert_eq!(parsed, expected);
}

#[rstest]
#[should_panic]
#[case(vec!["pacdef", "package", "search"], "regex missing")]
#[should_panic]
#[case(vec!["pacdef", "p", "s"], "can be either 'sync' or 'search'")]
#[should_panic]
#[case(vec!["pacdef", "group", "edit"], "group missing")]
fn arg_parsing_invalid(#[case] input: Vec<&str>, #[case] err_msg: &str) {
    build_cli().try_get_matches_from(input).expect(err_msg);
}

use pacdef_macros::Action;

/// All main actions the program can perform. Variants of the enum relate to
/// the different subcommands.
#[derive(Debug, Action)]
pub enum Actions {
    Clean,
    Edit,
    Import,
    List,
    New,
    Remove,
    Review,
    Search,
    Show,
    Sync,
    Unmanaged,
    Version,
}

pub enum MainActions {
    Group(GroupAction),
    Package(PackageAction),
    Version,
}

pub enum GroupAction {
    Edit,
    Import,
    List,
    New,
    Remove,
    Show,
}

pub enum PackageAction {
    Clean,
    Review,
    Search,
    Sync,
    Unmanaged,
}

use pacdef_macros::Action;

/// All actions the program can perform. Variants of the enum relate to
/// the different subcommands.
#[derive(Debug, Action)]
pub enum Actions {
    Clean,
    Completion,
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

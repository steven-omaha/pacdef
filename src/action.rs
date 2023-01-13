use pacdef_macro::Action;

#[derive(Debug, Action)]
pub(crate) enum Actions {
    Clean,
    Edit,
    Groups,
    Import,
    New,
    Remove,
    Search,
    Show,
    Sync,
    Unmanaged,
    Version,
}

use macros::Action;

#[derive(Debug, Action)]
pub enum Actions {
    Clean,
    Edit,
    Groups,
    Import,
    New,
    Remove,
    Review,
    Search,
    Show,
    Sync,
    Unmanaged,
    Version,
}

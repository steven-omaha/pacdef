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

// impl From<&str> for Actions {
//     fn from(value: &str) -> Self {
//         match value {

//         }
//     }
// }

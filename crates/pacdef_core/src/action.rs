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

#[derive(Debug)]
pub enum MainActions {
    Group(GroupAction),
    Package(PackageAction),
    Version,
}

#[derive(Debug, Clone)]
pub enum GroupAction {
    Edit,
    Import,
    List,
    New,
    Remove,
    Show,
}

#[derive(Debug, Clone)]
pub enum PackageAction {
    Clean,
    Review,
    Search,
    Sync,
    Unmanaged,
}

impl GroupAction {
    pub fn try_from(value: &str) -> Option<Self> {
        match value {
            "edit" => Some(Self::Edit),
            "import" => Some(Self::Import),
            "list" => Some(Self::List),
            "new" => Some(Self::New),
            "remove" => Some(Self::Remove),
            "show" => Some(Self::Show),
            _ => None,
        }
    }
}

impl PackageAction {
    pub fn try_from(value: &str) -> Option<Self> {
        match value {
            "clean" => Some(Self::Clean),
            "review" => Some(Self::Review),
            "search" => Some(Self::Search),
            "sync" => Some(Self::Sync),
            "unmanaged" => Some(Self::Unmanaged),
            _ => None,
        }
    }
}

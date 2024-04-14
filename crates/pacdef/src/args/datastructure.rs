#[derive(Debug, PartialEq)]
pub enum Arguments {
    Group(GroupAction),
    Package(PackageAction),
    Version,
}

#[derive(Debug, PartialEq)]
pub enum GroupAction {
    Edit(Groups),
    Export(Groups, OutputDir, Force),
    Import(Groups),
    List,
    New(Groups, Edit),
    Remove(Groups),
    Show(Groups),
}

#[derive(Debug, PartialEq)]
pub struct Files(pub Vec<String>);

#[derive(Debug, PartialEq)]
pub struct Groups(pub Vec<String>);

#[derive(Debug, PartialEq)]
pub enum PackageAction {
    Clean(Noconfirm),
    Review,
    Search(Regex),
    Sync(Noconfirm),
    Unmanaged,
}

#[derive(Debug, PartialEq)]
pub struct Regex(pub String);

#[derive(Debug, PartialEq)]
pub struct Edit(pub bool);

#[derive(Debug, PartialEq)]
pub struct Noconfirm(pub bool);

#[derive(Debug, PartialEq)]
pub struct Force(pub bool);

#[derive(Debug, PartialEq)]
pub struct OutputDir(pub Option<String>);

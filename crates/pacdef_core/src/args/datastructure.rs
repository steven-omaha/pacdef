#[derive(Debug)]
pub enum Arguments {
    Group(GroupAction),
    Package(PackageAction),
    Version,
}

#[derive(Debug)]
pub enum GroupAction {
    Edit(Groups),
    Export(Groups),
    Import(Groups),
    List,
    New(Groups, Edit),
    Remove(Groups),
    Show(Groups),
}

#[derive(Debug)]
pub struct Files(pub Vec<String>);

#[derive(Debug)]
pub struct Groups(pub Vec<String>);

#[derive(Debug)]
pub enum PackageAction {
    Clean(Noconfirm),
    Review,
    Search(Regex),
    Sync(Noconfirm),
    Unmanaged,
}

#[derive(Debug)]
pub struct Regex(pub String);

#[derive(Debug)]
pub struct Edit(pub bool);

#[derive(Debug)]
pub struct Noconfirm(pub bool);

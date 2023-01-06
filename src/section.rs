use std::{collections::HashSet, hash::Hash};

use crate::Package;

#[derive(Debug, Eq)]
pub struct Section {
    pub name: String,
    pub packages: HashSet<Package>,
}

impl Section {
    pub(crate) fn new(name: String, packages: HashSet<Package>) -> Self {
        Self { name, packages }
    }

    pub fn from_lines<'a>(iter: &mut impl Iterator<Item = &'a str>) -> Self {
        let name = iter.find(|line| line.starts_with('[')).unwrap().to_string();
        let packages = iter
            .take_while(|line| !line.starts_with('['))
            .filter(|line| !line.contains(char::is_alphabetic))
            .map(Package::from)
            .collect();

        Self::new(name, packages)
    }
}

impl Hash for Section {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Section {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

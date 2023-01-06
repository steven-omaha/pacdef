use std::collections::HashSet;
use std::fs::{read_to_string, File};
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::section::Section;
use crate::Package;

#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub sections: HashSet<Section>,
}

impl Group {
    pub fn load_from_dir() -> Result<HashSet<Self>> {
        let mut result = HashSet::new();

        let path = crate::path::get_pacdef_group_dir().context("getting pacdef group dir")?;
        for entry in path.read_dir().context("reading group dir")? {
            let file = entry.context("getting a file")?;
            let name = file.file_name();

            let group = Group::try_from(name)?;
            result.insert(group);
        }

        Ok(result)
    }
}

impl PartialOrd for Group {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(core::cmp::Ordering::Equal) => None,
            ord => ord,
        }
    }
}

impl Ord for Group {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl Hash for Group {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Group {
    fn assert_receiver_is_total_eq(&self) {}
}

impl<P> From<P> for Group
where
    P: AsRef<Path>,
{
    fn from(p: P) -> Self {
        let path = p.as_ref();
        let content = read_to_string(path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        let mut lines = content.lines().peekable();
        let mut sections = HashSet::new();

        while lines.peek().is_some() {
            let section = Section::from_lines(&mut lines);
            sections.insert(section);
        }

        Self { name, sections }
    }
}

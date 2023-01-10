use std::collections::HashSet;
use std::fs::read_to_string;
use std::hash::Hash;
use std::path::Path;

use anyhow::{Context, Result};

use crate::section::Section;

#[derive(Debug)]
pub struct Group {
    pub(crate) name: String,
    pub(crate) sections: HashSet<Section>,
}

impl Group {
    pub fn load_from_dir() -> Result<HashSet<Self>> {
        let mut result = HashSet::new();

        let path = crate::path::get_pacdef_group_dir().context("getting pacdef group dir")?;
        for entry in path.read_dir().context("reading group dir")? {
            let file = entry.context("getting group file")?;
            let path = file.path();

            let group =
                Group::try_from(&path).with_context(|| format!("reading group file {:?}", path))?;
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

impl Group {
    fn try_from<P>(p: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = p.as_ref();
        let content = read_to_string(path).context("reading file content")?;
        let name = path
            .file_name()
            .context("getting file name")?
            .to_string_lossy()
            .to_string();

        let mut lines = content.lines().peekable();
        let mut sections = HashSet::new();

        while lines.peek().is_some() {
            if let Ok(section) = Section::try_from_lines(&mut lines).context("reading section") {
                sections.insert(section);
            }
        }

        Ok(Self { name, sections })
    }
}

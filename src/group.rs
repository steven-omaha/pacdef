use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};

use anyhow::{anyhow, Context, Result};

use crate::Package;

#[derive(Debug)]
pub struct Group {
    pub name: String,
    pub packages: HashSet<Package>,
}

impl Group {
    pub fn load_from_dir() -> Result<HashSet<Self>> {
        let mut result = HashSet::new();
        let path = crate::path::get_pacdef_group_dir().context("getting pacdef group dir")?;
        for entry in path.read_dir().context("reading group dir")? {
            let file = entry.context("getting a file")?;
            let name = file.file_name();
            let f = File::open(file.path()).context("reading the file")?;
            let reader = BufReader::new(f);

            let packages = Package::from_lines(reader.lines());
            result.insert(Group {
                name: name
                    .into_string()
                    .map_err(|e| anyhow!("could not get group name, {e:?}"))?,
                packages,
            });
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
        self.name == other.name && self.packages == other.packages
    }
}

impl Eq for Group {
    fn assert_receiver_is_total_eq(&self) {}
}

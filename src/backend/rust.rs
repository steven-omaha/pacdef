use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;

use super::{Backend, Switches, Text};
use crate::{impl_backend_constants, Group, Package};

pub struct Rust {
    pub packages: HashSet<Package>,
}

const BINARY: Text = "cargo";
const SECTION: Text = "rust";
const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_REMOVE: Switches = &["uninstall"];

impl Backend for Rust {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> HashSet<Package> {
        let file = get_crates_file().unwrap();
        let content = read_to_string(file).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        extract_packages(json)
    }

    fn get_explicitly_installed_packages(&self) -> HashSet<Package> {
        self.get_all_installed_packages()
    }
}

fn extract_packages(json: Value) -> HashSet<Package> {
    json.get("installs")
        .unwrap()
        .as_object()
        .unwrap()
        .into_iter()
        .map(|(name, _)| name)
        .map(|name| name.split_whitespace().next().unwrap())
        .filter_map(Package::try_from)
        .collect()
}

impl Rust {
    pub fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }
}

impl Default for Rust {
    fn default() -> Self {
        Self::new()
    }
}

fn get_crates_file() -> Result<PathBuf> {
    let mut result = crate::path::get_home_dir()?;
    result.push(".cargo");
    result.push(".crates2.json");
    Ok(result)
}

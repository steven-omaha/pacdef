use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;

use anyhow::{Context, Result};
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

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        let file = get_crates_file().unwrap();
        let content = read_to_string(file).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        extract_packages(json)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        self.get_all_installed_packages()
    }
}

fn extract_packages(json: Value) -> Result<HashSet<Package>> {
    let result: HashSet<_> = json
        .get("installs")
        .context("get 'installs' field from json")?
        .as_object()
        .context("getting object")?
        .into_iter()
        .map(|(name, _)| name)
        .map(|name| {
            name.split_whitespace()
                .next()
                .expect("identifier is whitespace-delimited")
        })
        .map(|name| Package::try_from(name).expect("name is valid"))
        .collect();
    Ok(result)
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

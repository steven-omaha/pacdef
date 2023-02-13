use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::ExitStatus;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::backend::backend_trait::*;
use crate::{impl_backend_constants, Group, Package};

#[derive(Debug)]
pub(crate) struct Rust {
    pub(crate) packages: HashSet<Package>,
}

const BINARY: Text = "cargo";
const SECTION: Text = "rust";
const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_INFO: Switches = &["search", "--limit", "1"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[];
const SWITCHES_REMOVE: Switches = &["uninstall"];

impl Backend for Rust {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        let file = get_crates_file().context("getting path to crates file")?;
        let content = read_to_string(file).context("reading crates file")?;
        let json: Value =
            serde_json::from_str(&content).context("parsing JSON from crates file")?;
        extract_packages(&json).context("extracing packages from crates file")
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        self.get_all_installed_packages()
            .context("getting all installed packages")
    }

    fn make_dependency(&self, _: &[Package]) -> Result<ExitStatus> {
        unreachable!("cargo does not have unmanaged packages")
    }
}

fn extract_packages(json: &Value) -> Result<HashSet<Package>> {
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
    pub(crate) fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }
}

fn get_crates_file() -> Result<PathBuf> {
    let mut result = crate::path::get_home_dir().context("getting home dir")?;
    result.push(".cargo");
    result.push(".crates2.json");
    Ok(result)
}

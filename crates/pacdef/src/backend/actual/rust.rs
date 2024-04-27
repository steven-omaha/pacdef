use std::fs::read_to_string;
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rust {}
impl Rust {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for Rust {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for Rust {
    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            binary: "cargo".to_string(),
            section: "rust",
            switches_info: &["search", "--limit", "1"],
            switches_install: &["install"],
            switches_noconfirm: &[],
            switches_remove: &["uninstall"],
            switches_make_dependency: None,
        }
    }

    fn get_all_installed_packages(&self) -> Result<Packages> {
        let file = get_crates_file().context("getting path to crates file")?;

        let content = match read_to_string(file) {
            Ok(string) => string,
            Err(err) if err.kind() == NotFound => {
                log::warn!("no crates file found for cargo. Assuming no crates installed yet.");
                return Ok(Packages::new());
            }
            Err(err) => bail!(err),
        };

        let json: Value =
            serde_json::from_str(&content).context("parsing JSON from crates file")?;
        extract_packages(&json).context("extracting packages from crates file")
    }

    fn get_explicitly_installed_packages(&self) -> Result<Packages> {
        self.get_all_installed_packages()
            .context("getting all installed packages")
    }

    fn make_dependency(&self, _: &Packages) -> Result<()> {
        panic!("not supported by {}", self.backend_info().binary)
    }
}

fn extract_packages(json: &Value) -> Result<Packages> {
    let result: Packages = json
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

fn get_crates_file() -> Result<PathBuf> {
    let mut result = crate::path::get_cargo_home().context("getting cargo home dir")?;
    result.push(".crates2.json");
    Ok(result)
}

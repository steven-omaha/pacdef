use std::collections::BTreeSet;
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;
use std::{collections::BTreeMap, fs::read_to_string};

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cargo;

impl Backend for Cargo {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    type QueryInfo = ();
    type Modification = ();

    fn query_installed_packages(
        config: &Config,
    ) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        let file = get_crates_file().context("getting path to crates file")?;

        let content = match read_to_string(file) {
            Ok(string) => string,
            Err(err) if err.kind() == NotFound => {
                log::warn!("no crates file found for cargo. Assuming no crates installed yet.");
                return Ok(BTreeMap::new());
            }
            Err(err) => bail!(err),
        };

        extract_packages(&json).context("extracting packages from crates file")
    }

    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
    }

    fn modify_packages(
        packages: &BTreeMap<Self::PackageId, Self::Modification>,
        config: &Config,
    ) -> Result<()> {
        unimplemented!()
    }

    fn remove_packages(
        packages: &BTreeMap<Self::PackageId, Self::RemoveOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        todo!()
    }
}

impl Backend for Rust {
    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            binary: "cargo".to_string(),
            section: "rust",
            switches_info: &["search", "--limit", "1"],
            switches_install: &["install"],
            switches_no_confirm: &[],
            switches_remove: &["uninstall"],
            switches_make_dependency: None,
        }
    }

    fn get_installed_packages(&self) -> Result<Packages> {
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
        self.get_installed_packages()
            .context("getting all installed packages")
    }

    fn make_dependency(&self, _: &Packages) -> Result<()> {
        panic!("not supported by {}", self.backend_info().binary)
    }
}

fn extract_packages(json: &Value) -> Result<BTreeSet<String>> {
    let json: Value = serde_json::from_str(&content).context("parsing JSON from crates file")?;

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

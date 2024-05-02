use std::io::ErrorKind::NotFound;
use std::path::PathBuf;
use std::{collections::BTreeMap, fs::read_to_string};

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::cmd::run_args;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, derive_more::Display)]
pub struct Cargo;

impl Backend for Cargo {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    type QueryInfo = ();
    type Modification = ();

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        let file = get_crates_file().context("getting path to crates file")?;

        let contents = match read_to_string(file) {
            Ok(string) => string,
            Err(err) if err.kind() == NotFound => {
                log::warn!("no crates file found for cargo. Assuming no crates installed yet.");
                return Ok(BTreeMap::new());
            }
            Err(err) => bail!(err),
        };

        extract_packages(&contents).context("extracting packages from crates file")
    }

    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["cargo", "install"]
                .into_iter()
                .chain(packages.keys().map(String::as_str)),
        )
    }

    fn modify_packages(
        _: &BTreeMap<Self::PackageId, Self::Modification>,
        _: &Config,
    ) -> Result<()> {
        unimplemented!()
    }

    fn remove_packages(
        packages: &BTreeMap<Self::PackageId, Self::RemoveOptions>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["cargo", "uninstall"]
                .into_iter()
                .chain(packages.keys().map(String::as_str)),
        )
    }
}

fn extract_packages(contents: &String) -> Result<BTreeMap<String, ()>> {
    let json: Value = serde_json::from_str(contents).context("parsing JSON from crates file")?;

    let result: BTreeMap<String, ()> = json
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
        .map(|name| (name.to_string(), ()))
        .collect();

    Ok(result)
}

fn get_crates_file() -> Result<PathBuf> {
    let mut result = crate::path::get_cargo_home().context("getting cargo home dir")?;
    result.push(".crates2.json");
    Ok(result)
}

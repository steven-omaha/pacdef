use std::collections::BTreeMap;
use std::collections::BTreeSet;

use anyhow::Context;
use anyhow::Result;
use serde_json::Value;

use crate::cmd::run_args;
use crate::cmd::run_args_for_stdout;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, derive_more::Display)]
pub struct Pipx;

impl Backend for Pipx {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    type QueryInfo = ();
    type Modification = ();

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        let names =
            extract_package_names(run_args_for_stdout(["pipx", "list", "--json"].into_iter())?)?;

        Ok(names.into_iter().map(|x| (x, ())).collect())
    }

    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["pipx", "install"]
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
            ["pipx", "uninstall"]
                .into_iter()
                .chain(packages.keys().map(String::as_str)),
        )
    }
}

fn extract_package_names(stdout: String) -> Result<BTreeSet<String>> {
    let value: Value = serde_json::from_str(&stdout)?;

    let result = value["venvs"]
        .as_object()
        .context("getting inner json object")?
        .iter()
        .map(|(name, _)| name.clone())
        .collect();

    Ok(result)
}

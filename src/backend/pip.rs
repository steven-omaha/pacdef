use std::collections::BTreeMap;
use std::collections::BTreeSet;

use anyhow::Context;
use anyhow::Result;
use serde_json::Value;

use crate::cmd::command_found;
use crate::cmd::run_args;
use crate::cmd::run_args_for_stdout;
use crate::prelude::*;

#[derive(Debug, Copy, Clone, derive_more::Display)]
pub struct Pip;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PipQueryInfo {
    explicit: bool,
}

impl Backend for Pip {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    type QueryInfo = PipQueryInfo;
    type Modification = ();

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        if !command_found("pip") {
            return Ok(BTreeMap::new());
        }

        let all = extract_package_names(run_args_for_stdout(["pip", "list", "--format", "json"])?)?;
        let implicit = extract_package_names(run_args_for_stdout([
            "pip",
            "list",
            "--format",
            "json",
            "--not-required",
        ])?)?;

        let explicit = all.difference(&implicit);

        Ok(implicit
            .iter()
            .map(|x| (x.to_string(), PipQueryInfo { explicit: false }))
            .chain(explicit.map(|x| (x.to_string(), PipQueryInfo { explicit: true })))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["pip", "install"]
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
            ["pip", "uninstall"]
                .into_iter()
                .chain(packages.keys().map(String::as_str)),
        )
    }
}

fn extract_package_names(stdout: String) -> Result<BTreeSet<String>> {
    let value: Value = serde_json::from_str(&stdout)?;

    let result = value
        .as_array()
        .context("getting inner json array")?
        .iter()
        .map(|node| node["name"].as_str().expect("should always be a string"))
        .map(String::from)
        .collect();

    Ok(result)
}

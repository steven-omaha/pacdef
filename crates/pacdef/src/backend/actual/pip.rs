use std::collections::BTreeMap;
use std::collections::BTreeSet;

use anyhow::Context;
use anyhow::Result;
use serde_json::Value;

use crate::backend::root::run_args;
use crate::backend::root::run_args_for_stdout;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pip;

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
        let all = extract_package_names(run_args_for_stdout(
            ["pip", "list", "--format", "json"].into_iter(),
        )?)?;
        let implicit = extract_package_names(run_args_for_stdout(
            ["pip", "list", "--format", "json", "--not-required"].into_iter(),
        )?)?;

        let explicit = all.difference(&implicit);

        Ok(implicit
            .into_iter()
            .map(|x| (x, PipQueryInfo { explicit: false }))
            .chain(
                explicit
                    .into_iter()
                    .map(|x| (x.clone(), PipQueryInfo { explicit: true })),
            )
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

fn extract_pacdef_packages_pipx(value: Value) -> Result<Packages> {
    let result = value["venvs"]
        .as_object()
        .context("getting inner json object")?
        .iter()
        .map(|(name, _)| Package::from(name.as_str()))
        .collect();
    Ok(result)
}

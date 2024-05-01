use std::process::Command;

use anyhow::Result;
use regex::Regex;

use crate::backend::root::{run_args, run_args_for_stdout};
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Void {}
impl Void {
    pub fn new() -> Self {
        Self {}
    }

    fn no_confirm_flag(no_confirm: bool) -> Option<&'static str> {
        if no_confirm {
            Some("-y")
        } else {
            None
        }
    }
}
impl Default for Void {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MakeDependency;

impl Backend for Void {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    type QueryInfo = ();
    type Modification = MakeDependency;

    fn query_installed_packages(
        _: &Config,
    ) -> Result<std::collections::BTreeMap<Self::PackageId, Self::QueryInfo>> {
        let mut cmd = Command::new("xbps-query");
        cmd.args(["-l"]);
        let stdout = run_args_for_stdout(["xbps-query", "-l"].into_iter())?;

        // Removes the package status and description from output
        let re1 = Regex::new(r"^ii |^uu |^hr |^\?\? | .*")?;
        // Removes the package version from output
        let re2 = Regex::new(r"-[^-]*$")?;

        let packages = stdout
            .lines()
            .map(|line| {
                let mid_result = re1.replace_all(line, "");

                (re2.replace_all(&mid_result, "").to_string(), ())
            })
            .collect();

        Ok(packages)
    }

    fn install_packages(
        packages: &std::collections::BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["xbps-install", "-S"]
                .into_iter()
                .chain(Self::no_confirm_flag(no_confirm))
                .chain(packages.keys().map(String::as_str)),
        )
    }

    fn remove_packages(
        packages: &std::collections::BTreeMap<Self::PackageId, Self::RemoveOptions>,
        no_confirm: bool,
    ) -> Result<()> {
        run_args(
            ["xbps-remove", "-R"]
                .into_iter()
                .chain(Self::no_confirm_flag(no_confirm))
                .chain(packages.keys().map(String::as_str)),
        )
    }

    fn modify_packages(
        packages: &std::collections::BTreeMap<Self::PackageId, Self::Modification>,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["xbps-pkgdb", "-m", "auto"]
                .into_iter()
                .chain(packages.keys().map(String::as_str)),
        )
    }
}

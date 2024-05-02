use std::collections::BTreeMap;

use anyhow::Result;

use crate::cmd::{run_args, run_args_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, derive_more::Display)]
pub struct Dnf;

#[derive(Debug, Clone)]
pub struct DnfQueryInfo {
    user: bool,
}

#[derive(Debug, Clone)]
pub struct DnfInstallOptions {
    repo: Option<String>,
}

impl Backend for Dnf {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = DnfInstallOptions;
    type QueryInfo = DnfQueryInfo;
    type Modification = ();

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        let system_packages = run_args_for_stdout(
            [
                "dnf",
                "repoquery",
                "--installed",
                "--queryformat",
                "%{from_repo}/%{name}",
            ]
            .into_iter(),
        )?;
        let system_packages = system_packages.lines().map(parse_package);

        let user_packages = run_args_for_stdout(
            [
                "dnf",
                "repoquery",
                "--userinstalled",
                "--queryformat",
                "%{from_repo}/%{name}",
            ]
            .into_iter(),
        )?;
        let user_packages = user_packages.lines().map(parse_package);

        Ok(system_packages
            .map(|x| (x, DnfQueryInfo { user: false }))
            .chain(user_packages.map(|x| (x, DnfQueryInfo { user: true })))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        // add these two repositories as these are needed for many dependencies
        run_args(
            ["dnf", "install", "--repo", "updates", "--repo", "fedora"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(
                    packages
                        .iter()
                        .flat_map(|(package_id, options)| match &options.repo {
                            Some(repo) => vec![package_id, "--repo", repo.as_str()],
                            None => vec![package_id.as_str()],
                        }),
                ),
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
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["dnf", "remove"]
                .into_iter()
                .chain(Some("--assumeyes").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
        )
    }
}

fn parse_package(package: &str) -> String {
    // These repositories are ignored when storing the packages
    // as these are present by default on any sane fedora system
    if ["koji", "fedora", "updates", "anaconda", "@"]
        .iter()
        .any(|repo| package.contains(repo))
        && !package.contains("copr")
    {
        package
            .split('/')
            .nth(1)
            .expect("Cannot be empty!")
            .to_string()
    } else {
        package.to_string()
    }
}

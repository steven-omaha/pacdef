use alpm::{Alpm, PackageReason};
use anyhow::{Context, Result};
use std::collections::BTreeMap;

use crate::cmd::run_args;
use crate::prelude::*;

#[derive(Debug, Clone, derive_more::Display)]
pub struct Arch;

#[derive(Debug, Clone)]
pub struct ArchQueryInfo {
    reason: PackageReason,
}

pub struct ArchMakeImplicit;

impl Backend for Arch {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    type QueryInfo = ArchQueryInfo;
    type Modification = ArchMakeImplicit;

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        let alpm = Alpm::new("/", "/var/lib/pacman")
            .context("connecting to DB using expected default values")?;

        Ok(alpm
            .localdb()
            .pkgs()
            .iter()
            .map(|x| (x.name().to_string(), ArchQueryInfo { reason: x.reason() }))
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        run_args(
            [config.aur_helper.as_str(), "--sync"]
                .into_iter()
                .chain(Some("--no_confirm").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
        )
    }

    fn modify_packages(
        packages: &BTreeMap<Self::PackageId, Self::Modification>,
        config: &Config,
    ) -> Result<()> {
        run_args(
            [config.aur_helper.as_str(), "--database", "--asdeps"]
                .into_iter()
                .chain(packages.keys().map(String::as_str)),
        )
    }

    fn remove_packages(
        packages: &BTreeMap<Self::PackageId, Self::RemoveOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        run_args(
            [config.aur_helper.as_str(), "--remove", "--recursive"]
                .into_iter()
                .chain(config.aur_rm_args.iter().map(String::as_str))
                .chain(Some("--no_confirm").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
        )
    }
}

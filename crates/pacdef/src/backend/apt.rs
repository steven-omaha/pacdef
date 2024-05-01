use std::collections::BTreeMap;

use anyhow::Result;
use rust_apt::cache::{PackageSort, Sort};
use rust_apt::new_cache;

use crate::cmd::run_args;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Apt;

pub struct AptQueryInfo {
    explicit: bool,
}

pub struct AptMakeImplicit;

impl Backend for Apt {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    type QueryInfo = AptQueryInfo;
    type Modification = AptMakeImplicit;

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        let cache = new_cache!()?;

        let packages = cache.packages(&PackageSort {
            names: true,
            upgradable: Sort::Enable,
            virtual_pkgs: Sort::Enable,
            installed: Sort::Enable,
            auto_installed: Sort::Enable,
            auto_removable: Sort::Enable,
        })?;

        Ok(packages
            .map(|x| {
                (
                    x.name().to_string(),
                    AptQueryInfo {
                        explicit: !x.is_auto_installed(),
                    },
                )
            })
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["apt", "install"]
                .into_iter()
                .chain(Some("--yes").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
        )
    }

    fn modify_packages(
        packages: &BTreeMap<Self::PackageId, Self::Modification>,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["apt-mark", "auto"]
                .into_iter()
                .chain(packages.keys().map(String::as_str)),
        )
    }

    fn remove_packages(
        packages: &BTreeMap<Self::PackageId, Self::RemoveOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["apt", "remove"]
                .into_iter()
                .chain(Some("--yes").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
        )
    }
}

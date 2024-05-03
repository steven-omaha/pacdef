use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;

use crate::cmd::{command_found, run_args, run_args_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, derive_more::Display)]
pub struct Flatpak;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FlatpakQueryInfo {
    explicit: bool,
    systemwide: bool,
}

impl Backend for Flatpak {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    type QueryInfo = FlatpakQueryInfo;
    type Modification = ();

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        if !command_found("flatpak") {
            return Ok(BTreeMap::new());
        }

        let sys_explicit_btree = run_args_for_stdout(
            [
                "flatpak",
                "list",
                "--system",
                "--app",
                "--columns=application",
            ]
            .into_iter(),
        )?
        .lines()
        .map(String::from)
        .collect::<BTreeSet<_>>();
        let sys_all_btree = run_args_for_stdout(
            ["flatpak", "list", "--system", "--columns=application"].into_iter(),
        )?
        .lines()
        .map(String::from)
        .collect::<BTreeSet<_>>();

        let user_explicit_btree = run_args_for_stdout(
            [
                "flatpak",
                "list",
                "--user",
                "--app",
                "--columns=application",
            ]
            .into_iter(),
        )?
        .lines()
        .map(String::from)
        .collect::<BTreeSet<_>>();
        let user_all_btree = run_args_for_stdout(
            ["flatpak", "list", "--user", "--columns=application"].into_iter(),
        )?
        .lines()
        .map(String::from)
        .collect::<BTreeSet<_>>();

        let sys_explicit = sys_explicit_btree.iter().map(|x| {
            (
                x.clone(),
                FlatpakQueryInfo {
                    explicit: true,
                    systemwide: true,
                },
            )
        });
        let sys_implicit = sys_all_btree
            .iter()
            .filter(|x| !sys_explicit_btree.contains(*x))
            .map(|x| {
                (
                    x.clone(),
                    FlatpakQueryInfo {
                        explicit: false,
                        systemwide: true,
                    },
                )
            });
        let user_explicit = user_explicit_btree.iter().map(|x| {
            (
                x.clone(),
                FlatpakQueryInfo {
                    explicit: true,
                    systemwide: false,
                },
            )
        });
        let user_implicit = user_all_btree
            .iter()
            .filter(|x| !user_explicit_btree.contains(*x))
            .map(|x| {
                (
                    x.clone(),
                    FlatpakQueryInfo {
                        explicit: false,
                        systemwide: false,
                    },
                )
            });

        let all = sys_explicit
            .chain(sys_implicit)
            .chain(user_explicit)
            .chain(user_implicit)
            .collect();

        Ok(all)
    }

    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        run_args(
            [
                "flatpak",
                "install",
                if config.flatpak_systemwide {
                    "--system"
                } else {
                    "--user"
                },
            ]
            .into_iter()
            .chain(Some("--assumeyes").filter(|_| no_confirm))
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
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        run_args(
            [
                "flatpak",
                "uninstall",
                if config.flatpak_systemwide {
                    "--system"
                } else {
                    "--user"
                },
            ]
            .into_iter()
            .chain(Some("--assumeyes").filter(|_| no_confirm))
            .chain(packages.keys().map(String::as_str)),
        )
    }
}

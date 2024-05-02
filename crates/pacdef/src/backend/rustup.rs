use crate::cmd::run_args;
use crate::cmd::run_args_for_stdout;
use crate::prelude::*;
use anyhow::Result;
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, derive_more::Display)]
pub struct Rustup;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub enum RustupPackageId {
    Toolchain(String),
    /// Toolchain, Component
    #[display(fmt = "{}/{}", _0, _1)]
    Component(String, String),
}

impl Backend for Rustup {
    type PackageId = RustupPackageId;
    type RemoveOptions = ();
    type InstallOptions = ();
    type QueryInfo = ();
    type Modification = ();

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        let mut packages = BTreeMap::new();

        let toolchains_stdout = run_args_for_stdout(["rustup", "toolchain", "list"].into_iter())?;
        let toolchains = toolchains_stdout.lines().map(|x| {
            x.split(' ')
                .next()
                .expect("output shouldn't contain empty lines")
                .to_string()
        });

        for toolchain in toolchains {
            packages.insert(RustupPackageId::Toolchain(toolchain.clone()), ());

            let components_stdpout = run_args_for_stdout(
                [
                    "component",
                    "list",
                    "--installed",
                    "--toolchain",
                    toolchain.as_str(),
                ]
                .into_iter(),
            )?;

            for component in components_stdpout.lines() {
                packages.insert(
                    RustupPackageId::Component(component.to_string(), toolchain.to_string()),
                    (),
                );
            }
        }

        Ok(packages)
    }

    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        _: bool,
        _: &Config,
    ) -> Result<()> {
        for package_id in packages.keys() {
            match package_id {
                RustupPackageId::Toolchain(toolchain) => {
                    run_args(["rustup", "toolchain", "install", toolchain.as_str()].into_iter())?;
                }
                RustupPackageId::Component(toolchain, component) => {
                    run_args(
                        [
                            "rustup",
                            "component",
                            "add",
                            component.as_str(),
                            "--toolchain",
                            toolchain.as_str(),
                        ]
                        .into_iter(),
                    )?;
                }
            }
        }

        Ok(())
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
        for package_id in packages.keys() {
            match package_id {
                RustupPackageId::Toolchain(toolchain) => {
                    run_args(["rustup", "toolchain", "uninstall", toolchain.as_str()].into_iter())?;
                }
                RustupPackageId::Component(toolchain, component) => {
                    run_args(
                        [
                            "rustup",
                            "component",
                            "remove",
                            component.as_str(),
                            "--toolchain",
                            toolchain.as_str(),
                        ]
                        .into_iter(),
                    )?;
                }
            }
        }

        Ok(())
    }
}

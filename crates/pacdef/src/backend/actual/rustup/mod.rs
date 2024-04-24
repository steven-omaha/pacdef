mod helpers;
mod types;

use crate::cmd::run_external_command;
use crate::prelude::*;
use anyhow::{bail, Context, Result};
use std::process::Command;

use self::helpers::{
    group_components_by_toolchains, install_components, toolchain_of_component_was_already_removed,
};
use self::types::{Repotype, RustupPackage};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rustup {}
impl Rustup {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for Rustup {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for Rustup {
    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            binary: "rustup".to_string(),
            section: "rustup",
            switches_install: &["component", "add"],
            switches_info: &["component", "list", "--installed"],
            switches_noconfirm: &[],
            switches_remove: &["component", "remove"],
            switches_make_dependency: None,
        }
    }

    fn get_all_installed_packages(&self) -> Result<Packages> {
        let toolchains_vec = self
            .run_toolchain_command(Repotype::Toolchain.get_info_switches())
            .context("Getting installed toolchains")?;

        let toolchains: Packages = toolchains_vec
            .iter()
            .map(|name| ["toolchain", name].join("/").into())
            .collect();

        let components: Packages = self
            .run_component_command(Repotype::Component.get_info_switches(), &toolchains_vec)
            .context("Getting installed components")?
            .iter()
            .map(|name| ["component", name].join("/").into())
            .collect();

        let mut packages = Packages::new();

        packages.extend(toolchains);
        packages.extend(components);

        Ok(packages)
    }

    fn get_explicitly_installed_packages(&self) -> Result<Packages> {
        self.get_all_installed_packages()
            .context("Getting all installed packages")
    }

    fn make_dependency(&self, _: &[Package]) -> Result<()> {
        panic!("Not supported by {}", self.backend_info().binary)
    }

    fn install_packages(&self, packages: &[Package], _: bool) -> Result<()> {
        let packages = RustupPackage::from_pacdef_packages(packages)?;

        let (toolchains, components) =
            RustupPackage::sort_packages_into_toolchains_and_components(packages);

        self.install_toolchains(toolchains)?;
        self.install_components(components)?;

        Ok(())
    }

    fn remove_packages(&self, packages: &[Package], _: bool) -> Result<()> {
        let rustup_packages = RustupPackage::from_pacdef_packages(packages)?;

        let (toolchains, components) =
            RustupPackage::sort_packages_into_toolchains_and_components(rustup_packages);

        let removed_toolchains = self.remove_toolchains(toolchains)?;

        self.remove_components(components, removed_toolchains)?;

        Ok(())
    }
}

impl Rustup {
    fn run_component_command(&self, args: &[&str], toolchains: &[String]) -> Result<Vec<String>> {
        let mut val = Vec::new();

        for toolchain in toolchains {
            let mut cmd = Command::new(self.backend_info().binary);
            cmd.args(args).arg(toolchain);

            let output = String::from_utf8(cmd.output()?.stdout)?;

            for component in output.lines() {
                install_components(component, toolchain, &mut val);
            }
        }

        Ok(val)
    }

    fn run_toolchain_command(&self, args: &[&str]) -> Result<Vec<String>> {
        let mut cmd = Command::new(self.backend_info().binary);
        cmd.args(args);

        let output = String::from_utf8(cmd.output()?.stdout)?;

        let mut val = Vec::new();

        for line in output.lines() {
            let toolchain = line.split('-').next();
            match toolchain {
                Some(name) => val.push(name.to_string()),
                None => bail!("Toolchain name not provided!"),
            }
        }

        Ok(val)
    }

    fn install_toolchains(&self, toolchains: Vec<RustupPackage>) -> Result<()> {
        if toolchains.is_empty() {
            return Ok(());
        }
        let mut cmd = Command::new(self.backend_info().binary);
        cmd.args(Repotype::Toolchain.get_install_switches());

        for toolchain in toolchains {
            cmd.arg(&toolchain.toolchain);
        }

        run_external_command(cmd).context("installing toolchains")?;

        Ok(())
    }

    fn install_components(&self, components: Vec<RustupPackage>) -> Result<()> {
        if components.is_empty() {
            return Ok(());
        }

        let components_by_toolchain = group_components_by_toolchains(components);

        for components_for_one_toolchain in components_by_toolchain {
            let mut cmd = Command::new(self.backend_info().binary);
            cmd.args(Repotype::Component.get_install_switches());

            let the_toolchain = &components_for_one_toolchain
                .first()
                .expect("will have at least one element")
                .toolchain;

            cmd.arg(the_toolchain);

            for component_package in &components_for_one_toolchain {
                let actual_component = component_package
                    .component
                    .as_ref()
                    .expect("constructor makes sure this is Some");

                cmd.arg(actual_component);
            }

            run_external_command(cmd)
                .with_context(|| format!("installing [{components_for_one_toolchain:?}]"))?;
        }

        Ok(())
    }

    fn remove_toolchains(&self, toolchains: Vec<RustupPackage>) -> Result<Vec<String>> {
        let mut removed_toolchains = vec![];
        if !toolchains.is_empty() {
            let mut cmd = Command::new(self.backend_info().binary);
            cmd.args(Repotype::Toolchain.get_remove_switches());

            for toolchain_package in &toolchains {
                let name = toolchain_package.toolchain.as_str();
                cmd.arg(name);
                removed_toolchains.push(name.to_string());
            }

            run_external_command(cmd)
                .with_context(|| format!("removing toolchains [{toolchains:?}]"))?;
        }
        Ok(removed_toolchains)
    }

    fn remove_components(
        &self,
        components: Vec<RustupPackage>,
        removed_toolchains: Vec<String>,
    ) -> Result<()> {
        for component_package in components {
            let mut cmd = Command::new(self.backend_info().binary);
            cmd.args(Repotype::Component.get_remove_switches());

            if toolchain_of_component_was_already_removed(&removed_toolchains, &component_package) {
                continue;
            }

            cmd.arg(&component_package.toolchain);
            cmd.arg(
                component_package
                    .component
                    .as_ref()
                    .expect("the constructor ensures this cannot be None"),
            );

            run_external_command(cmd)
                .with_context(|| format!("removing component {component_package:?}"))?;
        }
        Ok(())
    }
}

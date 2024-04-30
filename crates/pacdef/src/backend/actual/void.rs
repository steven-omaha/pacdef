use std::process::Command;

use anyhow::Result;
use regex::Regex;

use crate::backend::root::build_base_command_with_privileges;
use crate::cmd::run_external_command;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Void {}
impl Void {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for Void {
    fn default() -> Self {
        Self::new()
    }
}

const INSTALL_BINARY: Text = "xbps-install";
const REMOVE_BINARY: Text = "xbps-remove";
const QUERY_BINARY: Text = "xbps-query";
const PKGDB_BINARY: Text = "xbps-pkgdb";

impl Backend for Void {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    typeQueryInfo = ;
    

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            binary: "xbps-install".to_string(),
            section: "void",
            switches_info: &[],
            switches_install: &["-S"],
            switches_noconfirm: &["-y"],
            switches_remove: &["-R"],
            switches_make_dependency: Some(&["-m", "auto"]),
        }
    }

    fn get_installed_packages(&self) -> Result<Packages> {
        // Removes the package status and description from output
        let re_str_1 = r"^ii |^uu |^hr |^\?\? | .*";
        // Removes the package version from output
        let re_str_2 = r"-[^-]*$";
        let re1 = Regex::new(re_str_1)?;
        let re2 = Regex::new(re_str_2)?;
        let mut cmd = Command::new(QUERY_BINARY);
        cmd.args(["-l"]);
        let output = String::from_utf8(cmd.output()?.stdout)?;

        let packages = output
            .lines()
            .map(|line| {
                let result = re1.replace_all(line, "");
                let result = re2.replace_all(&result, "");
                result.to_string().into()
            })
            .collect();
        Ok(packages)
    }

    fn get_explicitly_installed_packages(&self) -> Result<Packages> {
        // Removes the package version from output
        let re_str = r"-[^-]*$";
        let re = Regex::new(re_str)?;
        let mut cmd = Command::new(QUERY_BINARY);
        cmd.args(["-m"]);
        let output = String::from_utf8(cmd.output()?.stdout)?;

        let packages = output
            .lines()
            .map(|line| {
                let result = re.replace_all(line, "").to_string();
                result.into()
            })
            .collect();
        Ok(packages)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &Packages, noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = build_base_command_with_privileges(INSTALL_BINARY);
        cmd.args(backend_info.switches_install);

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    fn remove_packages(&self, packages: &Packages, noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = build_base_command_with_privileges(REMOVE_BINARY);
        cmd.args(backend_info.switches_remove);

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    fn make_dependency(&self, packages: &Packages) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = build_base_command_with_privileges(PKGDB_BINARY);
        cmd.args(
            backend_info
                .switches_make_dependency
                .expect("void should support make make dependency"),
        );

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Show information from package manager for package.
    fn show_package_info(&self, package: &Package) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new(QUERY_BINARY);
        cmd.args(backend_info.switches_info);
        cmd.arg(format!("{package}"));

        run_external_command(cmd)
    }
}

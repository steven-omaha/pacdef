use std::collections::HashSet;
use std::process::Command;

use anyhow::Result;
use regex::Regex;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::backend::root::build_base_command_with_privileges;
use crate::cmd::run_external_command;
use crate::{Group, Package};

#[derive(Debug, Clone)]
pub struct Void {
    pub packages: HashSet<Package>,
}
impl Void {
    pub fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }
}
impl Default for Void {
    fn default() -> Self {
        Self::new()
    }
}

const BINARY: Text = "xbps-install";
const INSTALL_BINARY: Text = "xbps-install";
const REMOVE_BINARY: Text = "xbps-remove";
const QUERY_BINARY: Text = "xbps-query";
const PKGDB_BINARY: Text = "xbps-pkgdb";
const SECTION: Text = "void";

const SWITCHES_INFO: Switches = &[];
const SWITCHES_INSTALL: Switches = &["-S"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &["-m", "auto"];
const SWITCHES_NOCONFIRM: Switches = &["-y"];
const SWITCHES_REMOVE: Switches = &["-R"];

const SUPPORTS_AS_DEPENDENCY: bool = true;

impl Backend for Void {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
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

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
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
    fn install_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let mut cmd = build_base_command_with_privileges(INSTALL_BINARY);
        cmd.args(self.get_switches_install());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    fn remove_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let mut cmd = build_base_command_with_privileges(REMOVE_BINARY);
        cmd.args(self.get_switches_remove());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    fn make_dependency(&self, packages: &[Package]) -> Result<()> {
        let mut cmd = build_base_command_with_privileges(PKGDB_BINARY);
        cmd.args(self.get_switches_make_dependency());

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Show information from package manager for package.
    fn show_package_info(&self, package: &Package) -> Result<()> {
        let mut cmd = Command::new(QUERY_BINARY);
        cmd.args(self.get_switches_info());
        cmd.arg(format!("{package}"));

        run_external_command(cmd)
    }
}

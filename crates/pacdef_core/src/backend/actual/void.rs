use core::panic;
use std::collections::HashSet;
use std::process::{Command, ExitStatus};

use anyhow::{Context, Result};
use regex::Regex;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::{Group, Package};

#[derive(Debug, Clone)]
pub struct Void {
    pub(crate) packages: HashSet<Package>,
}

const BINARY: Text = "xbps-install";
const INSTALL_BINARY: Text = "xbps-install";
const REMOVE_BINARY: Text = "xbps-remove";
const QUERY_BINARY: Text = "xbps-query";
const PKGDB_BINARY: Text = "xbps-pkgdb";
const SECTION: Text = "void";

const SWITCHES_INFO: Switches = &["-m"];
const SWITCHES_INSTALL: Switches = &["-S"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &["-m", "auto"];
const SWITCHES_NOCONFIRM: Switches = &["-y"];
const SWITCHES_REMOVE: Switches = &["-R"];

const SUPPORTS_AS_DEPENDENCY: bool = true;
impl Void {
    fn get_binary_info(&self) -> Text {
        QUERY_BINARY
    }

    fn get_binary_install(&self) -> Text {
        INSTALL_BINARY
    }

    fn get_binary_remove(&self) -> Text {
        REMOVE_BINARY
    }

    fn get_binary_make_dependency(&self) -> Text {
        PKGDB_BINARY
    }
}

impl Backend for Void {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        // Removes the package status and description from output
        let re_str_1 = r"^ii |^uu |^hr |^\?\? | .*";
        // Removes the package version from output
        let re_str_2 = r"-[^-]*$";
        let re1 = Regex::new(re_str_1)?;
        let re2 = Regex::new(re_str_2)?;
        let mut cmd = Command::new(self.get_binary_info());
        cmd.args(&["-l"]);
        let output = String::from_utf8(cmd.output()?.stdout)?;

        let packages: HashSet<Package> = output
            .lines()
            .map(|line| {
                let result = re1.replace_all(line, "").to_string();
                let result = re2.replace_all(&result, "").to_string();
                result.into()
            })
            .collect();
        Ok(packages)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        // Removes the package version from output
        let re_str = r"-[^-]*$";
        let re = Regex::new(re_str)?;
        let mut cmd = Command::new(self.get_binary_info());
        cmd.args(self.get_switches_info());
        let output = String::from_utf8(cmd.output()?.stdout)?;

        let packages: HashSet<Package> = output
            .lines()
            .map(|line| {
                let result = re.replace_all(line, "").to_string();
                result.into()
            })
            .collect();
        Ok(packages)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &[Package], noconfirm: bool) -> Result<ExitStatus> {
        let mut cmd = Command::new("sudo");
        cmd.arg(self.get_binary_install());
        cmd.args(self.get_switches_install());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command {cmd:?}"))
    }

    fn remove_packages(&self, packages: &[Package], noconfirm: bool) -> Result<ExitStatus> {
        let mut cmd = Command::new("sudo");
        cmd.arg(self.get_binary_remove());
        cmd.args(self.get_switches_remove());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }

    fn make_dependency(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new("sudo");
        cmd.arg(self.get_binary_make_dependency());
        cmd.args(self.get_switches_make_dependency());

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }
}

impl Void {
    pub fn new() -> Self {
        Void {
            packages: HashSet::new(),
        }
    }
}

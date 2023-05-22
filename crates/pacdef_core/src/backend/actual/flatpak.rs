use std::collections::HashSet;
use std::process::Command;
use std::process::ExitStatus;

use anyhow::{Context, Result};

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::{Group, Package};

#[derive(Debug, Clone)]
pub struct Flatpak {
    pub(crate) packages: HashSet<Package>,
    pub(crate) systemwide: bool,
}

const BINARY: Text = "flatpak";
const SECTION: Text = "flatpak";

const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_INFO: Switches = &["info"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[];
const SWITCHES_NOCONFIRM: Switches = &["--assumeyes"];
const SWITCHES_REMOVE: Switches = &["uninstall"];

const SUPPORTS_AS_DEPENDENCY: bool = false;

impl Backend for Flatpak {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        self.get_installed_packages(true)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        self.get_installed_packages(false)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &[Package], noconfirm: bool) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_install());
        cmd.args(self.get_switches_runtime());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command {cmd:?}"))
    }

    fn make_dependency(&self, _: &[Package]) -> Result<ExitStatus> {
        panic!("not supported by {}", BINARY)
    }

    /// Remove the specified packages.
    fn remove_packages(&self, packages: &[Package], noconfirm: bool) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_remove());
        cmd.args(self.get_switches_runtime());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }

    /// Show information from package manager for package.
    fn show_package_info(&self, package: &Package) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_info());
        cmd.args(self.get_switches_runtime());
        cmd.arg(format!("{package}"));
        cmd.status()
            .with_context(|| format!("running command {cmd:?}"))
    }
}

impl Flatpak {
    pub(crate) fn new() -> Self {
        Self {
            packages: HashSet::new(),
            systemwide: true,
        }
    }

    fn get_switches_runtime(&self) -> Switches {
        if self.systemwide {
            &[]
        } else {
            &["--user"]
        }
    }

    fn get_installed_packages(&self, include_implicit: bool) -> Result<HashSet<Package>> {
        let mut cmd = Command::new(BINARY);
        cmd.args(["list", "--columns=application"]);
        if !include_implicit {
            cmd.arg("--app");
        }
        if !self.systemwide {
            cmd.arg("--user");
        }

        let output = String::from_utf8(cmd.output()?.stdout)?;
        Ok(output
            .lines()
            .map(Package::from)
            .collect::<HashSet<Package>>())
    }
}

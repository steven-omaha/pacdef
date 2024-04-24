use std::process::Command;

use anyhow::Result;

use crate::cmd::run_external_command;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flatpak {
    pub systemwide: bool,
}
impl Flatpak {
    pub fn new(config: &Config) -> Self {
        Self {
            systemwide: config.flatpak_systemwide,
        }
    }

    fn get_switches_runtime(&self) -> Switches {
        if self.systemwide {
            &[]
        } else {
            &["--user"]
        }
    }

    fn get_installed_packages(&self, include_implicit: bool) -> Result<Packages> {
        let mut cmd = Command::new(self.backend_info().binary);
        cmd.args(["list", "--columns=application"]);
        if !include_implicit {
            cmd.arg("--app");
        }
        if !self.systemwide {
            cmd.arg("--user");
        }

        let output = String::from_utf8(cmd.output()?.stdout)?;
        Ok(output.lines().map(Package::from).collect::<Packages>())
    }
}

impl Backend for Flatpak {
    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            binary: "flatpak".to_string(),
            section: "flatpak",
            switches_info: &["info"],
            switches_install: &["install"],
            switches_noconfirm: &["--assumeyes"],
            switches_remove: &["uninstall"],
            switches_make_dependency: None,
        }
    }

    fn get_all_installed_packages(&self) -> Result<Packages> {
        self.get_installed_packages(true)
    }

    fn get_explicitly_installed_packages(&self) -> Result<Packages> {
        self.get_installed_packages(false)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new(backend_info.binary);
        cmd.args(backend_info.switches_install);
        cmd.args(self.get_switches_runtime());

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    fn make_dependency(&self, _: &[Package]) -> Result<()> {
        panic!("not supported by {}", self.backend_info().binary)
    }

    /// Remove the specified packages.
    fn remove_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new(backend_info.binary);
        cmd.args(backend_info.switches_remove);
        cmd.args(self.get_switches_runtime());

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Show information from package manager for package.
    fn show_package_info(&self, package: &Package) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new(backend_info.binary);
        cmd.args(backend_info.switches_info);
        cmd.args(self.get_switches_runtime());
        cmd.arg(format!("{package}"));

        run_external_command(cmd)
    }
}

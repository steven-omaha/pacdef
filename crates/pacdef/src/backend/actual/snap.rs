use anyhow::Result;
use std::process::Command;

use crate::backend::root::build_base_command_with_privileges;
use crate::cmd::run_external_command;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Snap {}

impl Snap {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Snap {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for Snap {
    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            binary: "snap".to_string(),
            section: "snap",
            switches_info: &["info"],
            switches_install: &["install"],
            switches_noconfirm: &[],
            switches_remove: &["remove"],
            switches_make_dependency: None, // Snap doesn't have the concept of dependencies
        }
    }

    fn get_all_installed_packages(&self) -> Result<Packages> {
        let mut cmd = Command::new(self.backend_info().binary);
        cmd.args(["list"]);

        let output = String::from_utf8(cmd.output()?.stdout)?;
        let mut packages = Packages::new();

        // Skip the first line which is the header
        for line in output.lines().skip(1) {
            // Format is: name  version  rev   tracking  publisher  notes
            if let Some(name) = line.split_whitespace().next() {
                packages.insert(Package::from(name.to_string()));
            }
        }

        Ok(packages)
    }

    fn get_explicitly_installed_packages(&self) -> Result<Packages> {
        // Snap doesn't differentiate between explicitly and implicitly installed packages
        // so we return all installed packages
        self.get_all_installed_packages()
    }

    fn make_dependency(&self, _packages: &Packages) -> Result<()> {
        // Snap doesn't have the concept of package dependencies in the same way as other package managers
        panic!("not supported by {}", self.backend_info().binary)
    }

    fn install_packages(&self, packages: &Packages, _noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();
        let mut cmd = build_base_command_with_privileges(&backend_info.binary);

        cmd.args(backend_info.switches_install);

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    fn remove_packages(&self, packages: &Packages, _noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();
        let mut cmd = build_base_command_with_privileges(&backend_info.binary);

        cmd.args(backend_info.switches_remove);

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }
}

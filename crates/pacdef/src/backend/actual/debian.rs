use anyhow::Result;
use rust_apt::cache::PackageSort;
use rust_apt::new_cache;

use crate::backend::root::build_base_command_with_privileges;
use crate::cmd::run_external_command;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Debian {}
impl Debian {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for Debian {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for Debian {
    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            binary: "apt".to_string(),
            section: "debian",
            switches_info: &["show"],
            switches_install: &["install"],
            switches_noconfirm: &["--yes"],
            switches_remove: &["remove"],
            switches_make_dependency: Some(&[]),
        }
    }

    fn get_all_installed_packages(&self) -> Result<Packages> {
        let cache = new_cache!()?;
        let sort = PackageSort::default().installed();

        let mut result = Packages::new();
        for pkg in cache.packages(&sort)? {
            result.insert(Package::from(pkg.name().to_string()));
        }
        Ok(result)
    }

    fn get_explicitly_installed_packages(&self) -> Result<Packages> {
        let cache = new_cache!()?;
        let sort = PackageSort::default().installed().manually_installed();

        let mut result = Packages::new();
        for pkg in cache.packages(&sort)? {
            result.insert(Package::from(pkg.name().to_string()));
        }
        Ok(result)
    }

    fn make_dependency(&self, packages: &Packages) -> Result<()> {
        let mut cmd = build_base_command_with_privileges("apt-mark");
        cmd.arg("auto");
        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &Packages, noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = build_base_command_with_privileges(&backend_info.binary);

        cmd.args(backend_info.switches_install);

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Remove the specified packages.
    fn remove_packages(&self, packages: &Packages, noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = build_base_command_with_privileges(&backend_info.binary);
        cmd.args(backend_info.switches_remove);

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }
}

use std::collections::HashSet;
use std::process::Command;

use alpm::Alpm;
use alpm::PackageReason::Explicit;
use anyhow::{Context, Result};

use crate::cmd::run_external_command;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Arch {
    pub binary: String,
    pub aur_rm_args: Vec<String>,
}
impl Arch {
    pub fn new(config: &Config) -> Self {
        Self {
            binary: config.aur_helper.clone(),
            aur_rm_args: config.aur_rm_args.clone(),
        }
    }
}

impl Backend for Arch {
    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            binary: self.binary.clone(),
            section: "arch",
            switches_info: &["--query", "--info"],
            switches_install: &["--sync"],
            switches_noconfirm: &["--noconfirm"],
            switches_remove: &["--remove", "--recursive"],
            switches_make_dependency: Some(&["--database", "--asdeps"]),
        }
    }

    fn get_all_installed_packages(&self) -> Result<Packages> {
        let alpm_packages = get_all_installed_packages_from_alpm()
            .context("getting all installed packages from alpm")?;

        let result = convert_to_pacdef_packages(alpm_packages);
        Ok(result)
    }

    fn get_explicitly_installed_packages(&self) -> Result<Packages> {
        let alpm_packages = get_explicitly_installed_packages_from_alpm()
            .context("getting all installed packages from alpm")?;
        let result = convert_to_pacdef_packages(alpm_packages);
        Ok(result)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &Packages, noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new(&self.binary);

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

        let mut cmd = Command::new(&self.binary);

        cmd.args(backend_info.switches_remove);
        cmd.args(&self.aur_rm_args);

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }
}

fn get_all_installed_packages_from_alpm() -> Result<HashSet<String>> {
    let db = get_db_handle().context("getting DB handle")?;
    let result = db
        .localdb()
        .pkgs()
        .iter()
        .map(|p| p.name().to_string())
        .collect();
    Ok(result)
}

fn get_explicitly_installed_packages_from_alpm() -> Result<HashSet<String>> {
    let db = get_db_handle().context("getting DB handle")?;
    let result = db
        .localdb()
        .pkgs()
        .iter()
        .filter(|p| p.reason() == Explicit)
        .map(|p| p.name().to_string())
        .collect();
    Ok(result)
}

fn convert_to_pacdef_packages(packages: HashSet<String>) -> Packages {
    packages.into_iter().map(Package::from).collect()
}

fn get_db_handle() -> Result<Alpm> {
    Alpm::new("/", "/var/lib/pacman").context("connecting to DB using expected default values")
}

use std::collections::HashSet;
use std::process::Command;

use alpm::Alpm;
use alpm::PackageReason::Explicit;
use anyhow::{Context, Result};

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::cmd::run_external_command;
use crate::Package;

#[derive(Debug, Clone)]
pub struct Arch {
    pub binary: String,
    pub aur_rm_args: Vec<String>,
    pub packages: HashSet<Package>,
}
impl Arch {
    pub fn new() -> Self {
        Self {
            binary: BINARY.to_string(),
            aur_rm_args: vec![],
            packages: HashSet::new(),
        }
    }
}
impl Default for Arch {
    fn default() -> Self {
        Self::new()
    }
}

const BINARY: Text = "pacman";
const SECTION: Text = "arch";

const SWITCHES_INFO: Switches = &["--query", "--info"];
const SWITCHES_INSTALL: Switches = &["--sync"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &["--database", "--asdeps"];
const SWITCHES_NOCONFIRM: Switches = &["--noconfirm"];
const SWITCHES_REMOVE: Switches = &["--remove", "--recursive"];

const SUPPORTS_AS_DEPENDENCY: bool = true;

impl Backend for Arch {
    impl_backend_constants!();

    fn get_binary(&self) -> Text {
        let r#box = self.binary.clone().into_boxed_str();
        Box::leak(r#box)
    }

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        let alpm_packages = get_all_installed_packages_from_alpm()
            .context("getting all installed packages from alpm")?;

        let result = convert_to_pacdef_packages(alpm_packages);
        Ok(result)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        let alpm_packages = get_explicitly_installed_packages_from_alpm()
            .context("getting all installed packages from alpm")?;
        let result = convert_to_pacdef_packages(alpm_packages);
        Ok(result)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let mut cmd = Command::new(&self.binary);

        cmd.args(self.get_switches_install());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Remove the specified packages.
    fn remove_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let mut cmd = Command::new(&self.binary);

        cmd.args(self.get_switches_remove());
        cmd.args(&self.aur_rm_args);

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
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

fn convert_to_pacdef_packages(packages: HashSet<String>) -> HashSet<Package> {
    packages.into_iter().map(Package::from).collect()
}

fn get_db_handle() -> Result<Alpm> {
    Alpm::new("/", "/var/lib/pacman").context("connecting to DB using expected default values")
}

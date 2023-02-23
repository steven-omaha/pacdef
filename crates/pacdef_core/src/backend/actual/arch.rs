use std::collections::HashSet;
use std::process::{Command, ExitStatus};

use alpm::Alpm;
use alpm::PackageReason::Explicit;
use anyhow::{Context, Result};

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::{impl_backend_constants, Group, Package};

#[derive(Debug, Clone)]
pub struct Arch {
    pub(crate) binary: String,
    pub(crate) aur_rm_args: Option<Vec<String>>,
    pub(crate) packages: HashSet<Package>,
}

const BINARY: Text = "pacman";
const SECTION: Text = "arch";

const SWITCHES_INFO: Switches = &["--query", "--info"];
const SWITCHES_INSTALL: Switches = &["--sync"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &["--database", "--asdeps"];
const SWITCHES_REMOVE: Switches = &["--remove", "--recursive"];

impl Backend for Arch {
    impl_backend_constants!();

    fn get_binary_inner(&self) -> Text {
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
    fn install_packages(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new(&self.binary);

        cmd.args(self.get_switches_install());

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command {cmd:?}"))
    }

    /// Remove the specified packages.
    fn remove_packages(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new(&self.binary);

        cmd.args(self.get_switches_remove());
        if let Some(rm_args) = &self.aur_rm_args {
            cmd.args(rm_args);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
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

impl Arch {
    pub(crate) fn new() -> Self {
        Self {
            binary: BINARY.to_string(),
            aur_rm_args: None,
            packages: HashSet::new(),
        }
    }
}

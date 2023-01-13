use std::collections::HashSet;

use alpm::Alpm;
use alpm::PackageReason::Explicit;
use anyhow::{Context, Result};

use crate::backend::backend_trait::*;
use crate::{impl_backend_constants, Group, Package};

pub(crate) struct Pacman {
    pub(crate) packages: HashSet<Package>,
}

const BINARY: Text = "paru";
const SECTION: Text = "pacman";
const SWITCHES_INSTALL: Switches = &["-S"];
const SWITCHES_REMOVE: Switches = &["-Rsn"];

impl Backend for Pacman {
    impl_backend_constants!();

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

impl Pacman {
    pub(crate) fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }
}

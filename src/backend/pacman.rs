use std::collections::HashSet;

use alpm::Alpm;
use alpm::PackageReason::Explicit;

use super::{Backend, Switches, Text};
use crate::{impl_backend_constants, Group, Package};

pub struct Pacman {
    pub packages: HashSet<Package>,
}

const BINARY: Text = "paru";
const SECTION: Text = "pacman";
const SWITCHES_INSTALL: Switches = &["-S"];
const SWITCHES_REMOVE: Switches = &["-Rsn"];

impl Backend for Pacman {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> HashSet<Package> {
        convert_to_pacdef_packages(get_all_installed_packages_from_alpm())
    }

    fn get_explicitly_installed_packages(&self) -> HashSet<Package> {
        convert_to_pacdef_packages(get_explicitly_installed_packages_from_alpm())
    }
}

fn get_all_installed_packages_from_alpm() -> HashSet<String> {
    let db = Alpm::new("/", "/var/lib/pacman").unwrap();
    db.localdb()
        .pkgs()
        .iter()
        .map(|p| p.name().to_string())
        .collect()
}

fn get_explicitly_installed_packages_from_alpm() -> HashSet<String> {
    let db = Alpm::new("/", "/var/lib/pacman").unwrap();
    db.localdb()
        .pkgs()
        .iter()
        .filter(|p| p.reason() == Explicit)
        .map(|p| p.name().to_string())
        .collect()
}

fn convert_to_pacdef_packages(packages: HashSet<String>) -> HashSet<Package> {
    packages.into_iter().filter_map(Package::try_from).collect()
}

impl Pacman {
    pub fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }
}

impl Default for Pacman {
    fn default() -> Self {
        Self::new()
    }
}

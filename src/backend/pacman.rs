use std::collections::HashSet;

use alpm::Alpm;
use alpm::PackageReason::Explicit;

use super::{Backend, Binary, Switches};
use crate::Package;

pub struct Pacman;

impl Backend for Pacman {
    const BINARY: Binary = "paru";
    const SWITCHES_INSTALL: Switches = &["-S"];
    const SWITCHES_REMOVE: Switches = &["-Rsn"];

    fn get_all_installed_packages() -> HashSet<Package> {
        convert_to_pacdef_packages(get_all_installed_packages_from_alpm())
    }

    fn get_explicitly_installed_packages() -> HashSet<Package> {
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
    packages.into_iter().map(Package::from).collect()
}

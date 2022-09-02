use crate::Package;
use alpm::Alpm;
use alpm::PackageReason::Explicit;
use std::collections::HashSet;

pub fn get_all_installed_packages() -> HashSet<Package> {
    convert_to_pacdef_packages(get_all_installed_packages_alpm())
}

fn get_all_installed_packages_alpm() -> HashSet<String> {
    let db = Alpm::new("/", "/var/lib/pacman").unwrap();
    db.localdb()
        .pkgs()
        .iter()
        .map(|p| p.name().to_string())
        .collect()
}

pub fn get_explicitly_installed_packages() -> HashSet<Package> {
    convert_to_pacdef_packages(get_explicitly_installed_packages_alpm())
}

fn get_explicitly_installed_packages_alpm() -> HashSet<String> {
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

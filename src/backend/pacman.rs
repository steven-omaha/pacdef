use std::collections::HashSet;
use std::os::unix::process::CommandExt;
use std::process::Command;

use alpm::Alpm;
use alpm::PackageReason::Explicit;

use super::Backend;
use crate::Package;

pub struct Pacman;

impl Backend for Pacman {
    const BINARY: &'static str = "paru";

    fn get_all_installed_packages() -> HashSet<Package> {
        convert_to_pacdef_packages(get_all_installed_packages_from_alpm())
    }

    fn get_explicitly_installed_packages() -> HashSet<Package> {
        convert_to_pacdef_packages(get_explicitly_installed_packages_from_alpm())
    }

    fn install_packages(packages: Vec<Package>) {
        let mut cmd = Command::new("paru");
        cmd.arg("-S");
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.exec();
    }

    fn remove_packages(packages: Vec<Package>) {
        let mut cmd = Command::new("paru");
        cmd.arg("-Rsn");
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.exec();
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

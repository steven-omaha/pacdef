mod pacman;

use std::collections::HashSet;
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::Package;

pub use pacman::Pacman;

pub trait Backend {
    /// The binary that should be called to run the associated package manager.
    const BINARY: &'static str;

    /// The switches that signals the `BINARY` that the packages should be installed.
    const SWITCH_INSTALL: &'static str;

    /// The switches that signals the `BINARY` that the packages should be removed.
    const SWITCH_REMOVE: &'static str;

    /// Get all packages that are installed in the system.
    fn get_all_installed_packages() -> HashSet<Package>;

    /// Get all packages that were installed in the system explicitly.
    fn get_explicitly_installed_packages() -> HashSet<Package>;

    /// Install the specified packages.
    fn install_packages(packages: Vec<Package>) {
        let mut cmd = Command::new(Self::BINARY);
        cmd.arg(Self::SWITCH_INSTALL);
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.exec();
    }

    /// Remove the specified packages.
    fn remove_packages(packages: Vec<Package>) {
        let mut cmd = Command::new(Self::BINARY);
        cmd.arg(Self::SWITCH_REMOVE);
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.exec();
    }
}

mod pacman;
mod rust;

use std::collections::HashSet;
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::Package;

pub use pacman::Pacman;
use rust::Rust;

pub fn get_backends() -> Vec<Box<dyn Backend>> {
    let pacman = Pacman;
    let rust = Rust;
    vec![Box::new(pacman), Box::new(rust)]
}

type Switches = &'static [&'static str];
type Binary = &'static str;

pub trait Backend {
    /// The binary that should be called to run the associated package manager.
    fn get_binary() -> Binary
    where
        Self: Sized;

    /// The switches that signals the `BINARY` that the packages should be installed.
    fn get_switches_install() -> Switches
    where
        Self: Sized;

    /// The switches that signals the `BINARY` that the packages should be removed.
    fn get_switches_remove() -> Switches
    where
        Self: Sized;

    /// Get all packages that are installed in the system.
    fn get_all_installed_packages() -> HashSet<Package>
    where
        Self: Sized;

    /// Get all packages that were installed in the system explicitly.
    fn get_explicitly_installed_packages() -> HashSet<Package>
    where
        Self: Sized;

    /// Install the specified packages.
    fn install_packages(packages: Vec<Package>)
    where
        Self: Sized,
    {
        let mut cmd = Command::new(Self::get_binary());
        cmd.args(Self::get_switches_install());
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.exec();
    }

    /// Remove the specified packages.
    fn remove_packages(packages: Vec<Package>)
    where
        Self: Sized,
    {
        let mut cmd = Command::new(Self::get_binary());
        cmd.args(Self::get_switches_remove());
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.exec();
    }
}

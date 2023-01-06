mod pacman;
mod rust;

use std::collections::HashSet;
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::Package;

pub use pacman::Pacman;
type Switches = &'static [&'static str];
type Text = &'static str;

pub enum Backends {
    Pacman,
    Rust,
}

pub trait Backend {
    /// The name that introduces the section in the group file
    const SECTION: Text;

    /// The binary that should be called to run the associated package manager.
    const BINARY: Text;

    /// The switches that signals the `BINARY` that the packages should be installed.
    const SWITCHES_INSTALL: Switches;

    /// The switches that signals the `BINARY` that the packages should be removed.
    const SWITCHES_REMOVE: Switches;

    /// Get all packages that are installed in the system.
    fn get_all_installed_packages() -> HashSet<Package>;

    /// Get all packages that were installed in the system explicitly.
    fn get_explicitly_installed_packages() -> HashSet<Package>;

    /// Install the specified packages.
    fn install_packages(packages: Vec<Package>) {
        let mut cmd = Command::new(Self::BINARY);
        cmd.args(Self::SWITCHES_INSTALL);
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.exec();
    }

    /// Remove the specified packages.
    fn remove_packages(packages: Vec<Package>) {
        let mut cmd = Command::new(Self::BINARY);
        cmd.args(Self::SWITCHES_REMOVE);
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.exec();
    }

    /// extract packages from its own section as read from group files
    fn extract_packages_from_group_file_content(content: &str) -> HashSet<Package> {
        content
            .lines()
            .skip_while(|line| !line.starts_with(&format!("[{}]", Self::SECTION)))
            .skip(1)
            .filter(|line| !line.starts_with('['))
            .fuse()
            .map(Package::from)
            .collect()
    }

    fn add_packages(&mut self, packages: HashSet<Package>);
}

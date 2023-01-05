mod pacman;

use std::collections::HashSet;

use crate::Package;

pub use pacman::Pacman;

pub trait Backend {
    /// The binary that should be called to run the associated package manager.
    const BINARY: &'static str;
    /// Get all packages that are installed in the system.
    fn get_all_installed_packages() -> HashSet<Package>;
    /// Get all packages that were installed in the system explicitly.
    fn get_explicitly_installed_packages() -> HashSet<Package>;
    /// Install the specified packages.
    fn install_packages(packages: Vec<Package>);
    /// Remove the specified packages.
    fn remove_packages(packages: Vec<Package>);
}

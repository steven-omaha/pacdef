mod pacman;

use std::collections::HashSet;

use crate::Package;

pub use pacman::Pacman;

pub trait Backend {
    /// Get all packages that are installed in the system.
    fn get_all_installed_packages() -> HashSet<Package>;
    /// Get all packages that were installed in the system explicitly.
    fn get_explicitly_installed_packages() -> HashSet<Package>;
}

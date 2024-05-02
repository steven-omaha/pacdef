use crate::prelude::*;
use anyhow::Result;

use std::collections::BTreeMap;

/// A type representing a users group files with all their packages
pub struct Groups {
    groups: BTreeMap<String, PackagesInstall>,
}
impl Groups {
    /// Convert to [`PackagesInstall`] using defaults for the backends' `InstallOptions`
    pub fn to_packages_install(&self) -> PackagesInstall {
        let mut packages = PackagesInstall::default();

        for group in self.groups.values() {
            packages.append(&mut group.clone());
        }

        packages
    }

    /// Returns `true` if no groups are contained
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Loads and parses the [`Groups`] struct from a users group files
    ///
    /// # Errors
    ///
    /// Returns an error if a parsing error is encountered in a found group file.
    pub fn load(_: &Config) -> Result<Self> {
        todo!()
    }
}

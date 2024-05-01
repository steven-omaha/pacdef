pub mod actual;
mod root;
pub mod todo_per_backend;

use std::collections::BTreeMap;

use crate::prelude::*;
use anyhow::Result;

/// A backend with its associated managed packages
pub struct ManagedBackend {
    /// All managed packages for this backend, i.e. all packages
    /// under the corresponding section in all group files.
    pub packages: Packages,
    pub any_backend: AnyBackend,
}

impl ManagedBackend {
    /// Get unmanaged packages
    ///
    /// # Errors
    ///
    /// Returns an error if the backend fails to get the explicitly installed packages.
    pub fn get_unmanaged_packages_sorted(&self) -> Result<Packages> {
        let installed = self
            .any_backend
            .get_explicitly_installed_packages()
            .context("could not get explicitly installed packages")?;

        let diff = installed.difference(&self.packages).cloned().collect();

        Ok(diff)
    }

    /// Get missing packages
    ///
    /// # Errors
    ///
    /// Returns an error if the backend fails to get the installed packages.
    pub fn get_missing_packages_sorted(&self) -> Result<Packages> {
        let installed = self
            .any_backend
            .get_installed_packages()
            .context("could not get installed packages")?;

        let diff = self.packages.difference(&installed).cloned().collect();

        Ok(diff)
    }
}

/// A trait to represent any package manager backend
#[enum_dispatch::enum_dispatch]
pub trait Backend {
    type PackageId;
    type InstallOptions;
    type RemoveOptions;
    type QueryInfo;
    type Modification;

    /// Query all packages that are installed in the system.
    ///
    /// # Errors
    ///
    /// This function shall return an error if the installed packages cannot be
    /// determined.
    fn query_installed_packages(
        config: &Config,
    ) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>>;

    /// Install the specified packages. If `no_confirm` is `true`, pass the corresponding
    /// switch to the package manager. Return the [`ExitStatus`] from the package manager.
    ///
    /// # Errors
    ///
    /// This function will return an error if the package manager cannot be run or it
    /// returns an error.
    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()>;

    /// Modify the packages as specified by [`Self::PackageModification`].
    ///
    /// This may not include installing or removing the package as [`Backend::install_packages()`]
    /// and [`Backend::remove_packages()`] exist for this purpose.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend fails to modify the packages as required.
    fn modify_packages(
        packages: &BTreeMap<Self::PackageId, Self::Modification>,
        config: &Config,
    ) -> Result<()>;

    /// Remove the specified packages.
    ///
    /// # Errors
    ///
    /// Returns an error if the external command fails.
    fn remove_packages(
        packages: &BTreeMap<Self::PackageId, Self::RemoveOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()>;
}

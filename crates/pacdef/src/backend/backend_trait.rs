use std::collections::BTreeMap;

use anyhow::Result;

use crate::prelude::*;

pub type Switches = &'static [&'static str];
pub type Text = &'static str;

/// A bundle of small of bits of info associated with a backend.
/// todo specialize to each backend and remove from here
pub struct BackendInfo {
    /// The binary name when calling the backend.
    pub binary: String,
    /// The name of the section in the group files.
    pub section: Text,
    /// CLI switches for the package manager to show information for
    /// packages.
    pub switches_info: Switches,
    /// CLI switches for the package manager to install packages.
    pub switches_install: Switches,
    /// CLI switches for the package manager to perform `sync` and `clean` without
    /// confirmation.
    pub switches_no_confirm: Switches,
    /// CLI switches for the package manager to remove packages.
    pub switches_remove: Switches,
    /// CLI switches for the package manager to mark packages as
    /// dependency. This is not supported by all package managers.
    pub switches_make_dependency: Option<Switches>,
}

/// The trait of a struct that is used as a backend.
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
    ) -> Result<()>;
}

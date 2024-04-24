use std::cmp::{Eq, Ord};
use std::collections::HashMap;
use std::hash::Hash;
use std::process::Command;

use anyhow::Result;

use crate::cmd::run_external_command;
use crate::prelude::*;

pub type Switches = &'static [&'static str];
pub type Text = &'static str;

/// A bundle of small of bits of info associated with a backend.
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
    pub switches_noconfirm: Switches,
    /// CLI switches for the package manager to remove packages.
    pub switches_remove: Switches,
    /// CLI switches for the package manager to mark packages as
    /// dependency. This is not supported by all package managers.
    pub switches_make_dependency: Option<Switches>,
}

/// The trait of a struct that is used as a backend.
#[enum_dispatch::enum_dispatch]
pub trait Backend {
    /// Return the [`BackendInfo`] associated with this backend.
    fn backend_info(&self) -> BackendInfo;

    fn supports_as_dependency(&self) -> bool {
        self.backend_info().switches_make_dependency.is_some()
    }

    /// Get all packages that are installed in the system.
    ///
    /// # Errors
    ///
    /// This function shall return an error if the installed packages cannot be
    /// determined.
    fn get_all_installed_packages(&self) -> Result<Packages>;

    /// Get all packages that were installed in the system explicitly.
    ///
    /// # Errors
    ///
    /// This function shall return an error if the explicitly installed packages
    /// cannot be determined.
    fn get_explicitly_installed_packages(&self) -> Result<Packages>;

    /// Assign each of the packages to an individual group by editing the
    /// group files.
    ///
    /// # Errors
    ///
    /// Returns an Error if any of the groups fails to save their given packages.
    fn assign_group(&self, to_assign: Vec<(Package, Group)>) -> Result<()> {
        let group_package_map = to_hashmap(to_assign);
        let section_header = format!("[{}]", self.backend_info().section);

        for (group, packages) in group_package_map {
            group.save_packages(&section_header, &packages)?;
        }

        Ok(())
    }

    /// Install the specified packages. If `noconfirm` is `true`, pass the corresponding
    /// switch to the package manager. Return the [`ExitStatus`] from the package manager.
    ///
    /// # Errors
    ///
    /// This function will return an error if the package manager cannot be run or it
    /// returns an error.
    fn install_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new(self.backend_info().binary);
        cmd.args(backend_info.switches_install);

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Mark the packages as non-explicit / dependency using the underlying
    /// package manager.
    ///
    /// # Panics
    ///
    /// This method shall panic when the backend does not support dependent packages.
    ///
    /// # Errors
    ///
    /// Returns an error if the external command fails.
    fn make_dependency(&self, packages: &[Package]) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new(backend_info.binary);

        if let Some(switches_make_dependency) = backend_info.switches_make_dependency {
            cmd.args(switches_make_dependency);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Remove the specified packages.
    ///
    /// # Errors
    ///
    /// Returns an error if the external command fails.
    fn remove_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new(backend_info.binary);
        cmd.args(backend_info.switches_remove);

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Show information from package manager for package.
    ///
    /// # Errors
    ///
    /// Returns an error if the external command fails.
    fn show_package_info(&self, package: &Package) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new(backend_info.binary);
        cmd.args(backend_info.switches_info);
        cmd.arg(format!("{package}"));

        run_external_command(cmd)
    }
}

/// For a vector of tuples containing a `V` and `K`, where a `K` may occur more than
/// once and each `V` exactly once, create a `HashMap` that associates each `K` with
/// a `Vec<V>`.
fn to_hashmap<K, V>(to_assign: Vec<(V, K)>) -> HashMap<K, Vec<V>>
where
    K: Hash + Eq,
    V: Ord,
{
    let mut map = HashMap::new();

    for (value, key) in to_assign {
        let inner: &mut Vec<V> = map.entry(key).or_default();
        inner.push(value);
    }

    for vecs in map.values_mut() {
        vecs.sort_unstable();
    }
    map
}

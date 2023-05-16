use std::any::Any;
use std::cmp::{Eq, Ord};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::process::{Command, ExitStatus};
use std::rc::Rc;

use anyhow::{Context, Result};

use crate::{Group, Package};

pub(in crate::backend) type Switches = &'static [&'static str];
pub(in crate::backend) type Text = &'static str;

/// The trait of a struct that is used as a backend.
pub trait Backend: Debug {
    /// Return the actual binary. Iff the backend supports different
    /// binaries, you will need to overwrite this implementation to return
    /// the binary that was loaded at runtime. See
    /// [`Backend::get_binary_default()`].
    fn get_binary(&self) -> Text {
        self.get_binary_default()
    }

    /// Return the default binary as defined in the constant of the module.
    /// See [`Backend::get_binary()`].
    fn get_binary_default(&self) -> Text;

    /// Get the name of the section in the group files.
    fn get_section(&self) -> Text;

    /// Get CLI switches for the package manager to show information for
    /// packages.
    fn get_switches_info(&self) -> Switches;

    /// Get CLI switches for the package manager to install packages.
    fn get_switches_install(&self) -> Switches;

    /// Get CLI switches for the package manager to perform `sync` and `clean` without
    /// confirmation.
    fn get_switches_noconfirm(&self) -> Switches;

    /// Get CLI switches for the package manager to remove packages.
    fn get_switches_remove(&self) -> Switches;

    /// Get CLI switches for the package manager to mark packages as
    /// dependency. This is not supported by all package managers. See
    /// [`Backend::supports_as_dependency`].
    fn get_switches_make_dependency(&self) -> Switches;

    /// Get CLI switches evaluated at runtime
    fn get_switches_runtime(&self) -> Switches {
        &[]
    }

    /// Load all packages from a set of groups. The backend will visit all groups,
    /// find its own section, and clone all packages into its own struct.
    fn load(&mut self, groups: &HashSet<Group>);

    /// Get all managed packages for this backend, i.e. all packages
    /// under the corresponding section in all group files.
    fn get_managed_packages(&self) -> &HashSet<Package>;

    /// Get all packages that are installed in the system.
    fn get_all_installed_packages(&self) -> Result<HashSet<Package>>;

    /// Get all packages that were installed in the system explicitly.
    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>>;

    /// Assign each of the packages to an individual group by editing the
    /// group files.
    fn assign_group(&self, to_assign: Vec<(Package, Rc<Group>)>) -> Result<()> {
        let group_package_map = to_hashmap(to_assign);
        let section_header = format!("[{}]", self.get_section());

        for (group, packages) in group_package_map {
            group.save_packages(&section_header, &packages)?;
        }

        Ok(())
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &[Package], noconfirm: bool) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_install());
        cmd.args(self.get_switches_runtime());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command {cmd:?}"))
    }

    /// Mark the packages as non-explicit / dependency using the underlying
    /// package manager.
    ///
    /// # Panics
    ///
    /// This method shall panic when the backend does not support depedent packages.
    fn make_dependency(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_make_dependency());
        cmd.args(self.get_switches_runtime());

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }

    /// Remove the specified packages.
    fn remove_packages(&self, packages: &[Package], noconfirm: bool) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_remove());
        cmd.args(self.get_switches_runtime());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }

    /// Get missing packages, sorted alphabetically.
    fn get_missing_packages_sorted(&self) -> Result<Vec<Package>> {
        let installed = self
            .get_all_installed_packages()
            .context("could not get installed packages")?;
        let managed = self.get_managed_packages();
        let mut diff: Vec<_> = managed.difference(&installed).cloned().collect();
        diff.sort_unstable();
        Ok(diff)
    }

    /// Show information from package manager for package.
    fn show_package_info(&self, package: &Package) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_info());
        cmd.args(self.get_switches_runtime());
        cmd.arg(format!("{package}"));
        cmd.status()
            .with_context(|| format!("running command {cmd:?}"))
    }

    /// Get unmanaged packages, sorted alphabetically.
    fn get_unmanaged_packages_sorted(&self) -> Result<Vec<Package>> {
        let installed = self
            .get_explicitly_installed_packages()
            .context("could not get explicitly installed packages")?;
        let required = self.get_managed_packages();
        let mut diff: Vec<_> = installed.difference(required).cloned().collect();
        diff.sort_unstable();
        Ok(diff)
    }

    /// Return a mutable reference to self as `Any`. Required for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Whether the underlying package manager supports dependency packages.
    fn supports_as_dependency(&self) -> bool;
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
        let inner = map.entry(key).or_insert(vec![]);
        inner.push(value);
    }

    for vecs in map.values_mut() {
        vecs.sort_unstable();
    }
    map
}

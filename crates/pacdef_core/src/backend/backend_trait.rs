use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
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

    /// Get CLI switches for the package manager to remove packages.
    fn get_switches_remove(&self) -> Switches;

    /// Get CLI switches for the package manager to mark packages as
    /// dependency. This is not supported by all package managers. See
    /// [`Backend::supports_as_dependency`].
    fn get_switches_make_dependency(&self) -> Switches;

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
        let group_package_map = get_group_packages_map(to_assign);
        let section_header = format!("[{}]", self.get_section());

        for (group, packages) in group_package_map {
            group.save_packages(&section_header, &packages)?;
        }

        Ok(())
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_install());
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.status()
            .with_context(|| format!("running command {cmd:?}"))
    }

    /// Mark the packages as non-explicit / dependency using the underlying
    /// package manager.
    fn make_dependency(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_make_dependency());
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }

    /// Remove the specified packages.
    fn remove_packages(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_remove());
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }

    /// TODO remove.
    fn extract_packages_from_group_file_content(&self, content: &str) -> HashSet<Package> {
        content
            .lines()
            .skip_while(|line| !line.starts_with(&format!("[{}]", self.get_section())))
            .skip(1)
            .filter(|line| !line.starts_with('['))
            .fuse()
            .filter_map(Package::try_from)
            .collect()
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

    /// TODO remove
    fn add_packages(&mut self, packages: HashSet<Package>);

    /// Show information from package manager for package.
    fn show_package_info(&self, package: &Package) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_info());
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

fn get_group_packages_map(
    to_assign: Vec<(Package, Rc<Group>)>,
) -> HashMap<Rc<Group>, Vec<Package>> {
    let mut group_package_map = HashMap::new();

    for (p, group) in to_assign {
        let inner = group_package_map.entry(group).or_insert(vec![]);
        inner.push(p);
    }

    for vecs in group_package_map.values_mut() {
        vecs.sort();
    }
    group_package_map
}

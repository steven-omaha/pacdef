use std::collections::HashSet;

use anyhow::Result;
use rust_apt::cache::PackageSort;
use rust_apt::new_cache;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::backend::root::build_base_command_with_privileges;
use crate::cmd::run_external_command;
use crate::{Group, Package};

#[derive(Debug, Clone)]
pub struct Debian {
    pub packages: HashSet<Package>,
}
impl Debian {
    pub fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }
}
impl Default for Debian {
    fn default() -> Self {
        Self::new()
    }
}

const BINARY: Text = "apt";
const SECTION: Text = "debian";

const SWITCHES_INFO: Switches = &["show"];
const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[]; // not needed
const SWITCHES_NOCONFIRM: Switches = &["--yes"];
const SWITCHES_REMOVE: Switches = &["remove"];

const SUPPORTS_AS_DEPENDENCY: bool = true;

impl Backend for Debian {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        let cache = new_cache!()?;
        let sort = PackageSort::default().installed();

        let mut result = HashSet::new();
        for pkg in cache.packages(&sort)? {
            result.insert(Package::from(pkg.name().to_string()));
        }
        Ok(result)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        let cache = new_cache!()?;
        let sort = PackageSort::default().installed().manually_installed();

        let mut result = HashSet::new();
        for pkg in cache.packages(&sort)? {
            result.insert(Package::from(pkg.name().to_string()));
        }
        Ok(result)
    }

    fn make_dependency(&self, packages: &[Package]) -> Result<()> {
        let mut cmd = build_base_command_with_privileges("apt-mark");
        cmd.arg("auto");
        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let mut cmd = build_base_command_with_privileges(self.get_binary());

        cmd.args(self.get_switches_install());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    /// Remove the specified packages.
    fn remove_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let mut cmd = build_base_command_with_privileges(self.get_binary());
        cmd.args(self.get_switches_remove());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }
}

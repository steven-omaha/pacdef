use std::collections::HashSet;
use std::process::Command;
use std::process::ExitStatus;

use anyhow::{Context, Result};
use rust_apt::cache::PackageSort;
use rust_apt::new_cache;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::{impl_backend_constants, Group, Package};

#[derive(Debug, Clone)]
pub struct Debian {
    pub(crate) packages: HashSet<Package>,
}

const BINARY: Text = "apt";
const SECTION: Text = "debian";

const SWITCHES_INFO: Switches = &["show"];
const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[]; // not needed
const SWITCHES_REMOVE: Switches = &["remove"];

const SUPPORTS_AS_DEPENDENCY: bool = true;

impl Backend for Debian {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        let cache = new_cache!()?;
        let sort = PackageSort::default().installed();

        let mut result = HashSet::new();
        for pkg in cache.packages(&sort) {
            result.insert(Package::from(pkg.name().to_string()));
        }
        Ok(result)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        let cache = new_cache!()?;
        let sort = PackageSort::default().installed().manually_installed();

        let mut result = HashSet::new();
        for pkg in cache.packages(&sort) {
            result.insert(Package::from(pkg.name().to_string()));
        }
        Ok(result)
    }

    fn make_dependency(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new("apt-mark");
        cmd.arg("auto");
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }
}

impl Debian {
    pub(crate) fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }
}

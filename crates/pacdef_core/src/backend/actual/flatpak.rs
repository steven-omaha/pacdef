use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::process::ExitStatus;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::{impl_backend_constants, Group, Package};

#[derive(Debug, Clone)]
pub struct Flatpak {
    pub(crate) packages: HashSet<Package>,
}

const BINARY: Text = "flatpak";
const SECTION: Text = "flatpak";

const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_INFO: Switches = &[];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[];
const SWITCHES_NOCONFIRM: Switches = &[];
const SWITCHES_REMOVE: Switches = &["uninstall"];

const SUPPORTS_AS_DEPENDENCY: bool = false;

impl Backend for Flatpak {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        Ok(HashSet::new())
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        Ok(HashSet::new())
    }

    fn make_dependency(&self, _: &[Package]) -> Result<ExitStatus> {
        panic!("not supported by {}", BINARY)
    }
}

impl Flatpak {
    pub(crate) fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }
}

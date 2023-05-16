use std::collections::HashSet;
use std::process::Command;
use std::process::ExitStatus;

use anyhow::Result;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::{impl_backend_constants, Group, Package};

#[derive(Debug, Clone)]
pub struct Flatpak {
    pub(crate) packages: HashSet<Package>,
    pub(crate) systemwide: bool,
}

const BINARY: Text = "flatpak";
const SECTION: Text = "flatpak";

const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_INFO: Switches = &["info"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[];
const SWITCHES_NOCONFIRM: Switches = &["--assumeyes"];
const SWITCHES_REMOVE: Switches = &["uninstall"];

const SUPPORTS_AS_DEPENDENCY: bool = false;

impl Backend for Flatpak {
    impl_backend_constants!();

    fn get_switches_runtime(&self) -> Switches {
        if self.systemwide {
            &[]
        } else {
            &["--user"]
        }
    }

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        self.get_installed_packages(true)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        self.get_installed_packages(false)
    }

    fn make_dependency(&self, _: &[Package]) -> Result<ExitStatus> {
        panic!("not supported by {}", BINARY)
    }
}

impl Flatpak {
    pub(crate) fn new() -> Self {
        Self {
            packages: HashSet::new(),
            systemwide: true,
        }
    }

    fn get_installed_packages(&self, include_implicit: bool) -> Result<HashSet<Package>> {
        let mut cmd = Command::new(BINARY);
        cmd.args(&["list", "--columns=application"]);
        if !include_implicit {
            cmd.arg("--app");
        }
        if !self.systemwide {
            cmd.arg("--user");
        }

        let output = String::from_utf8(cmd.output()?.stdout)?;
        Ok(
            output.lines()
                  .skip(1)
                  .map(|pkg| Package::from(pkg))
                  .collect::<HashSet<Package>>()
        )
    }
}

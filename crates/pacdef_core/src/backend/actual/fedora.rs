use core::panic;
use std::collections::HashSet;
use std::process::{Command, ExitStatus};

use anyhow::{Context, Result};
use regex::Regex;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::{Group, Package};

#[derive(Debug, Clone)]
pub struct Fedora {
    pub(crate) packages: HashSet<Package>,
}

const BINARY: Text = "dnf";
const SECTION: Text = "fedora";

const SWITCHES_INFO: Switches = &["list", "--installed"];
const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[];
const SWITCHES_NOCONFIRM: Switches = &["--assumeyes"];
const SWITCHES_REMOVE: Switches = &["remove"];

const SUPPORTS_AS_DEPENDENCY: bool = true;

impl Backend for Fedora {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        let re_str = r"^[0-9A-Za-z_-]*.";
        let re = Regex::new(re_str)?;

        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_info());
        let output = String::from_utf8(cmd.output()?.stdout)?;

        let packages: HashSet<Package> = output
            .lines()
            .map(|line| {
                let result = re
                    .find(
                        line.split_whitespace()
                            .next()
                            .expect("First word cannot be empty!"),
                    )
                    .expect("Not a valid package name!");
                let mut result = result.as_str().to_string();
                result.pop();
                result.into()
            })
            .collect();
        Ok(packages)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        let re_str = r"^(([A-Za-z_]*[0-9]*)-)*";
        let re = Regex::new(re_str)?;

        let mut cmd = Command::new(self.get_binary());
        cmd.args(&["history", "userinstalled"]);
        let output = String::from_utf8(cmd.output()?.stdout)?;

        let packages: HashSet<Package> = output
            .lines()
            .skip(1)
            .map(|line| {
                let word = re.find(line).expect("Not a valid package name");
                let mut word = word.as_str().to_string();
                word.pop();
                let pack = word.rsplit_once('-').map_or(word.clone(), |(pack, term)| {
                    let mut value = true;
                    for i in term.chars() {
                        if !i.is_numeric() {
                            value = false;
                            break;
                        }
                    }
                    if !value {
                        pack.to_string() + "-" + term
                    } else {
                        pack.to_string()
                    }
                });
                pack.into()
            })
            .collect();
        Ok(packages)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &[Package], noconfirm: bool) -> Result<ExitStatus> {
        let mut cmd = Command::new("sudo");
        cmd.arg(self.get_binary());
        cmd.args(self.get_switches_install());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command {cmd:?}"))
    }

    fn remove_packages(&self, packages: &[Package], noconfirm: bool) -> Result<ExitStatus> {
        let mut cmd = Command::new("sudo");
        cmd.arg(self.get_binary());
        cmd.args(self.get_switches_remove());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }

    fn make_dependency(&self, _: &[Package]) -> Result<ExitStatus> {
        panic!("Not supported by the package manager!")
    }
}

impl Fedora {
    pub fn new() -> Self {
        Fedora {
            packages: HashSet::new(),
        }
    }
}

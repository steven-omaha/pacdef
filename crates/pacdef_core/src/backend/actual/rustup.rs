use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::{Group, Package};
use anyhow::Context;
use core::panic;
use std::collections::HashSet;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Rustup {
    pub(crate) packages: HashSet<Package>,
}

const BINARY: Text = "rustup";
const SECTION: Text = "rustup";

const SWITCHES_INSTALL: Switches = &["component", "add"];
const SWITCHES_INFO: Switches = &["component", "list", "--installed"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[];
const SWITCHES_NOCONFIRM: Switches = &[];
const SWITCHES_REMOVE: Switches = &["component", "remove"];

const SUPPORTS_AS_DEPENDENCY: bool = false;

impl Backend for Rustup {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> anyhow::Result<HashSet<Package>> {
        let mut cmd = Command::new(self.get_binary());
        let packages: HashSet<Package> = run_rustup_command(&mut cmd, SWITCHES_INFO)
            .context("Getting installed components")?
            .iter()
            .map(|name| ["component/", name].join("").into())
            .collect();
        Ok(packages)
    }

    fn get_explicitly_installed_packages(&self) -> anyhow::Result<HashSet<Package>> {
        self.get_all_installed_packages()
            .context("Getting all installed packages")
    }

    fn make_dependency(&self, _: &[Package]) -> anyhow::Result<std::process::ExitStatus> {
        panic!("Not supported by {}", BINARY)
    }
}

fn run_rustup_command(cmd: &mut Command, args: &[&str]) -> Result<Vec<String>, anyhow::Error> {
    cmd.args(args);
    let output = String::from_utf8(cmd.output()?.stdout)?;
    let mut val = Vec::new();
    for i in output.lines() {
        let mut it = i.splitn(3, "-");
        let component = it.next().expect("Component name is empty!");
        match component {
            "cargo" | "rustfmt" | "clippy" | "miri" | "rls" | "rustc" => {
                val.push(component.to_string());
            }
            _ => {
                val.push(
                    component.to_string()
                        + "-"
                        + it.next().expect("No such component is managed by rustup"),
                );
            }
        }
    }
    Ok(val)
}

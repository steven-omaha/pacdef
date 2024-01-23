use std::collections::HashSet;
use std::process::Command;
use std::process::ExitStatus;

use anyhow::Context;
use anyhow::Result;
use serde_json::Value;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::{Group, Package};

#[derive(Debug, Clone)]
pub struct Python {
    pub(crate) binary: String,
    pub(crate) packages: HashSet<Package>,
}

const BINARY: Text = "pip";
const SECTION: Text = "python";

const SWITCHES_INFO: Switches = &["show"];
const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[]; // not needed
const SWITCHES_NOCONFIRM: Switches = &[]; // not needed
const SWITCHES_REMOVE: Switches = &["uninstall"];

const SUPPORTS_AS_DEPENDENCY: bool = false;

impl Backend for Python {
    impl_backend_constants!();

    fn get_binary(&self) -> Text {
        let r#box = self.binary.clone().into_boxed_str();
        Box::leak(r#box)
    }

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        let mut cmd = Command::new(self.get_binary());
        let output = run_pip_command(&mut cmd, self.get_switches_runtime())?;

        extract_pacdef_packages(output)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        let mut cmd = Command::new(self.get_binary());
        let output = run_pip_command(
            &mut cmd,
            &["list", "--format", "json", "--not-required", "--user"],
        )?;

        extract_pacdef_packages(output)
    }

    fn make_dependency(&self, _packages: &[Package]) -> Result<ExitStatus> {
        panic!("not supported by {}", BINARY)
    }
}

fn run_pip_command(cmd: &mut Command, args: &[&str]) -> Result<Value> {
    cmd.args(args);
    let output = String::from_utf8(cmd.output()?.stdout)?;
    let val: Value = serde_json::from_str(&output)?;
    Ok(val)
}

impl Python {
    pub(crate) fn new() -> Self {
        Self {
            binary: BINARY.to_string(),
            packages: HashSet::new(),
        }
    }

    fn get_switches_runtime(&self) -> Switches {
        if self.get_binary().eq("pip") {
            &["list", "--format", "json", "--user"]
        } else {
            &["list", "--json"]
        }
    }
}

fn extract_pacdef_packages(value: Value) -> Result<HashSet<Package>> {
    let result = value
        .as_array()
        .context("getting inner json array")?
        .iter()
        .map(|node| node["name"].as_str().expect("should always be a string"))
        .map(Package::from)
        .collect();
    Ok(result)
}

use std::collections::HashSet;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use serde_json::Value;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::Package;

macro_rules! ERROR{
    ($bin:expr) => {
        panic!("Cannot use {} for package management in python. Please use a valid package manager like pip or pipx.", $bin)
    };
}

#[derive(Debug, Clone)]
pub struct Python {
    pub binary: String,
    pub packages: HashSet<Package>,
}
impl Python {
    pub fn new() -> Self {
        Self {
            binary: BINARY.to_string(),
            packages: HashSet::new(),
        }
    }

    fn get_switches_runtime(&self) -> Switches {
        match self.get_binary() {
            "pip" => &["list", "--format", "json", "--not-required", "--user"],
            "pipx" => &["list", "--json"],
            _ => ERROR!(self.get_binary()),
        }
    }
    fn get_switches_explicit(&self) -> Switches {
        match self.get_binary() {
            "pip" => &["list", "--format", "json", "--user"],
            "pipx" => &["list", "--json"],
            _ => ERROR!(self.get_binary()),
        }
    }

    fn extract_packages(&self, output: Value) -> Result<HashSet<Package>> {
        match self.get_binary() {
            "pip" => extract_pacdef_packages(output),
            "pipx" => extract_pacdef_packages_pipx(output),
            _ => ERROR!(self.get_binary()),
        }
    }
}
impl Default for Python {
    fn default() -> Self {
        Self::new()
    }
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
        self.extract_packages(output)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        let mut cmd = Command::new(self.get_binary());
        let output = run_pip_command(&mut cmd, self.get_switches_explicit())?;
        self.extract_packages(output)
    }

    fn make_dependency(&self, _packages: &[Package]) -> Result<()> {
        panic!("not supported by {}", BINARY)
    }
}

fn run_pip_command(cmd: &mut Command, args: &[&str]) -> Result<Value> {
    cmd.args(args);
    let output = String::from_utf8(cmd.output()?.stdout)?;
    let val: Value = serde_json::from_str(&output)?;
    Ok(val)
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

fn extract_pacdef_packages_pipx(value: Value) -> Result<HashSet<Package>> {
    let result = value["venvs"]
        .as_object()
        .context("getting inner json object")?
        .iter()
        .map(|(name, _)| Package::from(name.as_str()))
        .collect();
    Ok(result)
}

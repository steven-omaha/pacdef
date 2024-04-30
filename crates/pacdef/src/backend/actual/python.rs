use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use serde_json::Value;

use crate::prelude::*;

macro_rules! ERROR{
    ($bin:expr) => {
        panic!("Cannot use {} for package management in python. Please use a valid package manager like pip or pipx.", $bin)
    };
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Python {
    pub binary: String,
}
impl Python {
    pub fn new(config: &Config) -> Self {
        Self {
            binary: config.pip_binary.to_string(),
        }
    }

    fn get_switches_runtime(&self) -> Switches {
        match self.backend_info().binary.as_str() {
            "pip" => &["list", "--format", "json", "--not-required", "--user"],
            "pipx" => &["list", "--json"],
            _ => ERROR!(self.backend_info().binary),
        }
    }
    fn get_switches_explicit(&self) -> Switches {
        match self.backend_info().binary.as_str() {
            "pip" => &["list", "--format", "json", "--user"],
            "pipx" => &["list", "--json"],
            _ => ERROR!(self.backend_info().binary),
        }
    }

    fn extract_packages(&self, output: Value) -> Result<Packages> {
        match self.backend_info().binary.as_str() {
            "pip" => extract_pacdef_packages(output),
            "pipx" => extract_pacdef_packages_pipx(output),
            _ => ERROR!(self.backend_info().binary),
        }
    }
}

impl Backend for Python {
    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            binary: self.binary.clone(),
            section: "python",
            switches_info: &["show"],
            switches_install: &["install"],
            switches_noconfirm: &[],
            switches_remove: &["uninstall"],
            switches_make_dependency: None,
        }
    }

    fn get_installed_packages(&self) -> Result<Packages> {
        let mut cmd = Command::new(self.backend_info().binary);
        let output = run_pip_command(&mut cmd, self.get_switches_runtime())?;
        self.extract_packages(output)
    }

    fn get_explicitly_installed_packages(&self) -> Result<Packages> {
        let mut cmd = Command::new(self.backend_info().binary);
        let output = run_pip_command(&mut cmd, self.get_switches_explicit())?;
        self.extract_packages(output)
    }

    fn make_dependency(&self, _packages: &Packages) -> Result<()> {
        panic!("not supported by {}", self.binary)
    }
}

fn run_pip_command(cmd: &mut Command, args: &[&str]) -> Result<Value> {
    cmd.args(args);
    let output = String::from_utf8(cmd.output()?.stdout)?;
    let val: Value = serde_json::from_str(&output)?;
    Ok(val)
}

fn extract_pacdef_packages(value: Value) -> Result<Packages> {
    let result = value
        .as_array()
        .context("getting inner json array")?
        .iter()
        .map(|node| node["name"].as_str().expect("should always be a string"))
        .map(Package::from)
        .collect();
    Ok(result)
}

fn extract_pacdef_packages_pipx(value: Value) -> Result<Packages> {
    let result = value["venvs"]
        .as_object()
        .context("getting inner json object")?
        .iter()
        .map(|(name, _)| Package::from(name.as_str()))
        .collect();
    Ok(result)
}

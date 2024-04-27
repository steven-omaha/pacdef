use std::process::Command;

use anyhow::Result;

use crate::cmd::run_external_command;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fedora {}
impl Fedora {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for Fedora {
    fn default() -> Self {
        Self::new()
    }
}

/// These repositories are ignored when storing the packages
/// as these are present by default on any sane fedora system
const DEFAULT_REPOS: [&str; 5] = ["koji", "fedora", "updates", "anaconda", "@"];

/// These switches are responsible for
/// getting the packages explicitly installed by the user
const SWITCHES_FETCH_USER: Switches = &[
    "repoquery",
    "--userinstalled",
    "--queryformat",
    "%{from_repo}/%{name}",
];

/// These switches are responsible for
/// getting all the packages installed on the system
const SWITCHES_FETCH_GLOBAL: Switches = &[
    "repoquery",
    "--installed",
    "--queryformat",
    "%{from_repo}/%{name}",
];

impl Backend for Fedora {
    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            binary: "dnf".to_string(),
            section: "fedora",
            switches_info: &["info"],
            switches_install: &["install"],
            switches_noconfirm: &["--assumeyes"],
            switches_remove: &["remove"],
            switches_make_dependency: None,
        }
    }

    fn get_all_installed_packages(&self) -> Result<Packages> {
        let mut cmd = Command::new(self.backend_info().binary);
        cmd.args(SWITCHES_FETCH_GLOBAL);

        let output = String::from_utf8(cmd.output()?.stdout)?;
        let packages = output.lines().map(create_package).collect();

        Ok(packages)
    }

    fn get_explicitly_installed_packages(&self) -> Result<Packages> {
        let mut cmd = Command::new(self.backend_info().binary);
        cmd.args(SWITCHES_FETCH_USER);

        let output = String::from_utf8(cmd.output()?.stdout)?;
        let packages = output.lines().map(create_package).collect();

        Ok(packages)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &Packages, noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new("sudo");
        cmd.arg(backend_info.binary);
        cmd.args(backend_info.switches_install);

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(&p.name);
            if let Some(repo) = p.repo.as_ref() {
                cmd.args(["--repo", repo]);
            }
        }

        // add these two repositories as these are needed for many dependencies
        cmd.args(["--repo", "updates"]);
        cmd.args(["--repo", "fedora"]);

        run_external_command(cmd)
    }

    /// Show information from package manager for package.
    fn remove_packages(&self, packages: &Packages, noconfirm: bool) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new("sudo");
        cmd.arg(backend_info.binary);
        cmd.args(backend_info.switches_remove);

        if noconfirm {
            cmd.args(backend_info.switches_noconfirm);
        }

        for p in packages {
            cmd.arg(&p.name);
        }

        run_external_command(cmd)
    }

    fn show_package_info(&self, package: &Package) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = Command::new(backend_info.binary);
        cmd.args(backend_info.switches_info);
        cmd.arg(&package.name);

        run_external_command(cmd)
    }

    fn make_dependency(&self, _: &Packages) -> Result<()> {
        panic!("Not supported by the package manager!")
    }
}

fn create_package(package: &str) -> Package {
    if DEFAULT_REPOS.iter().any(|repo| package.contains(repo)) && !package.contains("copr") {
        let package = package.split('/').nth(1).expect("Cannot be empty!");
        package.into()
    } else {
        package.into()
    }
}

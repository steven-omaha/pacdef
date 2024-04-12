use std::collections::HashSet;
use std::process::Command;

use anyhow::Result;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::cmd::run_external_command;
use crate::{Group, Package};

#[derive(Debug, Clone)]
pub struct Fedora {
    pub(crate) packages: HashSet<Package>,
}

const BINARY: Text = "dnf";
const SECTION: Text = "fedora";

const SWITCHES_INFO: Switches = &["info"];
const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[];
const SWITCHES_NOCONFIRM: Switches = &["--assumeyes"];
const SWITCHES_REMOVE: Switches = &["remove"];

const SUPPORTS_AS_DEPENDENCY: bool = false;

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

/// These repositories are ignored when storing the packages
/// as these are present by default on any sane fedora system
const DEFAULT_REPOS: [&str; 5] = ["koji", "fedora", "updates", "anaconda", "@"];

impl Backend for Fedora {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(SWITCHES_FETCH_GLOBAL);

        let output = String::from_utf8(cmd.output()?.stdout)?;
        let packages = output.lines().map(create_package).collect();

        Ok(packages)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(SWITCHES_FETCH_USER);

        let output = String::from_utf8(cmd.output()?.stdout)?;
        let packages = output.lines().map(create_package).collect();

        Ok(packages)
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let mut cmd = Command::new("sudo");
        cmd.arg(self.get_binary());
        cmd.args(self.get_switches_install());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
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
    fn remove_packages(&self, packages: &[Package], noconfirm: bool) -> Result<()> {
        let mut cmd = Command::new("sudo");
        cmd.arg(self.get_binary());
        cmd.args(self.get_switches_remove());

        if noconfirm {
            cmd.args(self.get_switches_noconfirm());
        }

        for p in packages {
            cmd.arg(&p.name);
        }

        run_external_command(cmd)
    }

    fn show_package_info(&self, package: &Package) -> Result<()> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_info());
        cmd.arg(&package.name);

        run_external_command(cmd)
    }

    fn make_dependency(&self, _: &[Package]) -> Result<()> {
        panic!("Not supported by the package manager!")
    }
}

impl Fedora {
    pub fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
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
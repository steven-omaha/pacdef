use std::{collections::HashSet, process::Command};

use super::{Backend, Switches, Text};
use crate::{impl_backend_constants, Group, Package};

pub struct Rust {
    pub packages: HashSet<Package>,
}

const BINARY: Text = "cargo";
const SECTION: Text = "rust";
const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_REMOVE: Switches = &["uninstall"];

impl Backend for Rust {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> HashSet<Package> {
        extract_packages_names(&run_cargo_install_list())
            .map(Package::from)
            .collect()
    }

    fn get_explicitly_installed_packages(&self) -> HashSet<Package> {
        self.get_all_installed_packages()
    }
}

fn run_cargo_install_list() -> String {
    let stdout = Command::new("cargo")
        .args(["install", "--list"])
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(stdout).unwrap()
}

fn extract_packages_names(output: &str) -> impl Iterator<Item = String> + '_ {
    output
        .lines()
        .filter(|line| !line.starts_with(char::is_whitespace))
        .map(|line| line.split_whitespace().next().unwrap().to_owned())
}

impl Rust {
    pub fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }
}

impl Default for Rust {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::extract_packages_names;

    #[test]
    fn test_extract_packages() {
        const OUTPUT: &str = "cargo-audit v0.17.4:
    cargo-audit
cargo-cache v0.8.3:
    cargo-cache
cargo-criterion v1.1.0:
    cargo-criterion
cargo-update v11.1.1:
    cargo-install-update
    cargo-install-update-config
flamegraph v0.6.2:
    cargo-flamegraph
    flamegraph
topgrade v10.1.2 (/home/ratajc72/tmp/topgrade):
    topgrade
wthrr v0.6.1:
    wthrr";
        let extracted: Vec<String> = extract_packages_names(OUTPUT).collect();
        assert_eq!(
            &extracted,
            &[
                "cargo-audit",
                "cargo-cache",
                "cargo-criterion",
                "cargo-update",
                "flamegraph",
                "topgrade",
                "wthrr"
            ]
        );
    }
}

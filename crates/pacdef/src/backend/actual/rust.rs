use std::collections::HashSet;
use std::fs::read_to_string;
use std::io::ErrorKind::NotFound;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::Package;

#[derive(Debug, Clone)]
pub struct Rust {
    pub packages: HashSet<Package>,
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

const BINARY: Text = "cargo";
const SECTION: Text = "rust";

const SWITCHES_INSTALL: Switches = &["install"];
const SWITCHES_INFO: Switches = &["search", "--limit", "1"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[];
const SWITCHES_NOCONFIRM: Switches = &[]; // not needed
const SWITCHES_REMOVE: Switches = &["uninstall"];

const SUPPORTS_AS_DEPENDENCY: bool = false;

impl Backend for Rust {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        let file = get_crates_file().context("getting path to crates file")?;

        let content = match read_to_string(file) {
            Ok(string) => string,
            Err(err) if err.kind() == NotFound => {
                eprintln!(
                    "WARNING: no crates file found for cargo. Assuming no crates installed yet."
                );
                return Ok(HashSet::new());
            }
            Err(err) => bail!(err),
        };

        let json: Value =
            serde_json::from_str(&content).context("parsing JSON from crates file")?;
        extract_packages(&json).context("extracting packages from crates file")
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        self.get_all_installed_packages()
            .context("getting all installed packages")
    }

    fn make_dependency(&self, _: &[Package]) -> Result<()> {
        panic!("not supported by {}", BINARY)
    }
}

fn extract_packages(json: &Value) -> Result<HashSet<Package>> {
    let result: HashSet<_> = json
        .get("installs")
        .context("get 'installs' field from json")?
        .as_object()
        .context("getting object")?
        .into_iter()
        .map(|(name, _)| name)
        .map(|name| {
            name.split_whitespace()
                .next()
                .expect("identifier is whitespace-delimited")
        })
        .map(|name| Package::try_from(name).expect("name is valid"))
        .collect();
    Ok(result)
}

fn get_crates_file() -> Result<PathBuf> {
    let mut result = crate::path::get_cargo_home().context("getting cargo home dir")?;
    result.push(".crates2.json");
    Ok(result)
}

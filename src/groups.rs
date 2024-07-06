use crate::prelude::*;
use anyhow::{Context, Result};
use path_absolutize::Absolutize;
use walkdir::WalkDir;

use std::{
    collections::BTreeMap,
    fs::{create_dir, read_to_string},
    ops::{Deref, DerefMut},
    path::Path,
    str::FromStr,
};

/// A type representing a users group files with all their packages
#[derive(Default)]
pub struct Groups(BTreeMap<String, PackagesInstall>);

impl Groups {
    /// Convert to [`PackagesInstall`] using defaults for the backends' `InstallOptions`
    pub fn to_packages_install(&self) -> PackagesInstall {
        let mut packages = PackagesInstall::default();

        for group in self.values() {
            packages.append(&mut group.clone());
        }

        packages
    }

    /// Return a new, empty Group
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Loads and parses the [`Groups`] struct from a users group files
    ///
    /// # Errors
    ///
    /// Returns an error if a parsing error is encountered in a found group file.
    ///
    /// # Panics
    ///
    /// Cannot panic on Linux, since all path types will always be convertible to
    /// valid Rust strings.
    pub fn load(group_dir: &Path, _config: &Config) -> Result<Self> {
        let mut groups = Self::new();

        if !group_dir.is_dir() {
            // other direcotories were already created with the config file
            create_dir(group_dir).context("group dir does not exist, creating one")?;
        }

        let mut files = vec![];

        for file in WalkDir::new(group_dir).follow_links(true) {
            let path = file?.path().absolutize_from(group_dir)?.to_path_buf();
            files.push(path);
        }

        for group_file in files.iter().filter(|path| path.is_file()) {
            let group = parse_group(group_file).context(format!(
                "Failed to parse the group file {}",
                group_file.to_string_lossy()
            ))?;

            let group_name = get_relative_path(group_file.as_path(), group_dir)
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .reduce(|mut a, b| {
                    a.push('/');
                    a.push_str(&b);
                    a
                })
                .expect("must have at least one element")
                .to_string();
            groups.insert(group_name, group);
        }

        Ok(groups)
    }
}

/// Parse the group and sections from the group file
fn parse_group(group_file: &Path) -> Result<PackagesInstall> {
    let content = read_to_string(group_file).context("reading file contents")?;

    let mut backends = PackagesIds::new();
    let mut prev_backend: Option<AnyBackend> = None;
    for (count, line) in content
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim_start().is_empty() && !line.trim_start().starts_with('#'))
    {
        if line.starts_with('[') {
            let backend = line
                .strip_prefix('[')
                .expect("Won't error")
                .strip_suffix(']')
                .ok_or(Error::InvalidBackend(line.to_string()))
                .context(format!("Invalid backend {}", line))?;

            prev_backend = AnyBackend::from_str(backend).ok();
        } else {
            let package = line
                .split_once('#')
                .map_or(line, |value| value.0)
                .trim()
                .to_string();

            prev_backend
                .map(|backend| backends.insert_backend_package(backend, package))
                .ok_or(Error::InvalidBackend(format!(
                    "Failed to parse {line} at line number {count} in file {}",
                    group_file.to_string_lossy()
                )))??;
        }
    }

    Ok(PackagesInstall::from_packages_ids_defaults(&backends))
}

impl Deref for Groups {
    type Target = BTreeMap<String, PackagesInstall>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Groups {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

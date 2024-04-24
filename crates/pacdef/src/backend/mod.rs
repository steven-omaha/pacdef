pub mod actual;
pub mod backend_trait;
mod root;
pub mod todo_per_backend;

use std::fmt::Display;

use crate::prelude::*;
use anyhow::{Context, Result};

/// A backend with its associated managed packages
pub struct ManagedBackend {
    /// All managed packages for this backend, i.e. all packages
    /// under the corresponding section in all group files.
    pub packages: Packages,
    pub any_backend: AnyBackend,
}

impl ManagedBackend {
    /// Get unmanaged packages, sorted alphabetically.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend fails to get the explicitly installed packages.
    pub fn get_unmanaged_packages_sorted(&self) -> Result<Vec<Package>> {
        let installed = self
            .any_backend
            .get_explicitly_installed_packages()
            .context("could not get explicitly installed packages")?;
        let mut diff: Vec<_> = installed.difference(&self.packages).cloned().collect();
        diff.sort_unstable();
        Ok(diff)
    }

    /// Get missing packages, sorted alphabetically.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend fails to get the installed packages.
    pub fn get_missing_packages_sorted(&self) -> Result<Vec<Package>> {
        let installed = self
            .any_backend
            .get_all_installed_packages()
            .context("could not get installed packages")?;
        let mut diff: Vec<_> = self.packages.difference(&installed).cloned().collect();
        diff.sort_unstable();
        Ok(diff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[enum_dispatch::enum_dispatch(Backend)]
pub enum AnyBackend {
    #[cfg(feature = "arch")]
    Arch(actual::arch::Arch),
    #[cfg(feature = "debian")]
    Debian(actual::debian::Debian),
    Flatpak(Flatpak),
    Fedora(Fedora),
    Python(Python),
    Rust(Rust),
    Rustup(Rustup),
    Void(Void),
}
impl AnyBackend {
    /// Returns an iterator of every variant of backend.
    pub fn all(config: &Config) -> impl Iterator<Item = Self> {
        vec![
            #[cfg(feature = "arch")]
            Self::Arch(actual::arch::Arch::new(config)),
            #[cfg(feature = "debian")]
            Self::Debian(actual::debian::Debian::new()),
            Self::Flatpak(Flatpak::new(config)),
            Self::Fedora(Fedora::new()),
            Self::Python(Python::new(config)),
            Self::Rust(Rust::new()),
            Self::Rustup(Rustup::new()),
            Self::Void(Void::new()),
        ]
        .into_iter()
    }

    pub fn from_section(section: &str, config: &Config) -> Result<Self> {
        match section {
            #[cfg(feature = "arch")]
            "arch" => Ok(Self::Arch(actual::arch::Arch::new(config))),
            #[cfg(feature = "debian")]
            "debian" => Ok(Self::Debian(actual::debian::Debian::new())),
            "flatpak" => Ok(Self::Flatpak(Flatpak::new(config))),
            "fedora" => Ok(Self::Fedora(Fedora::new())),
            "python" => Ok(Self::Python(Python::new(config))),
            "rust" => Ok(Self::Rust(Rust::new())),
            "rustup" => Ok(Self::Rustup(Rustup::new())),
            "void" => Ok(Self::Void(Void::new())),
            _ => Err(anyhow::anyhow!(
                "no matching backend for the section: {section}"
            )),
        }
    }
}
impl Display for AnyBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.backend_info().section)
    }
}

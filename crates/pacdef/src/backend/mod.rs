pub mod actual;
pub mod backend_trait;
mod root;
pub mod todo_per_backend;

use std::fmt::Display;

use crate::prelude::*;
use anyhow::Result;

/// A backend with its associated managed packages
pub struct ManagedBackend {
    /// All managed packages for this backend, i.e. all packages
    /// under the corresponding section in all group files.
    pub packages: Packages,
    pub any_backend: AnyBackend,
}

impl ManagedBackend {
    /// Get unmanaged packages
    ///
    /// # Errors
    ///
    /// Returns an error if the backend fails to get the explicitly installed packages.
    pub fn get_unmanaged_packages_sorted(&self) -> Result<Packages> {
        let installed = self
            .any_backend
            .get_explicitly_installed_packages()
            .context("could not get explicitly installed packages")?;

        let diff = installed.difference(&self.packages).cloned().collect();

        Ok(diff)
    }

    /// Get missing packages
    ///
    /// # Errors
    ///
    /// Returns an error if the backend fails to get the installed packages.
    pub fn get_missing_packages_sorted(&self) -> Result<Packages> {
        let installed = self
            .any_backend
            .get_installed_packages()
            .context("could not get installed packages")?;

        let diff = self.packages.difference(&installed).cloned().collect();

        Ok(diff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[enum_dispatch::enum_dispatch(Backend)]
pub enum AnyBackend {
    Arch(actual::arch::Arch),
    Debian(actual::apt::Debian),
    Flatpak(Flatpak),
    Fedora(Fedora),
    Python(Python),
    Rust(Rust),
    Rustup(Rustup),
    Void(Xbps),
}
impl AnyBackend {
    /// Returns an iterator of every variant of backend.
    pub fn all(config: &Config) -> impl Iterator<Item = Self> {
        vec![
            Self::Arch(actual::arch::Arch::new(config)),
            Self::Debian(actual::apt::Debian::new()),
            Self::Flatpak(Flatpak::new(config)),
            Self::Fedora(Fedora::new()),
            Self::Python(Python::new(config)),
            Self::Rust(Rust::new()),
            Self::Rustup(Rustup::new()),
            Self::Void(Xbps::new()),
        ]
        .into_iter()
    }

    pub fn from_section(section: &str, config: &Config) -> Result<Self> {
        match section {
            "arch" => Ok(Self::Arch(actual::arch::Arch::new(config))),
            "debian" => Ok(Self::Debian(actual::apt::Debian::new())),
            "flatpak" => Ok(Self::Flatpak(Flatpak::new(config))),
            "fedora" => Ok(Self::Fedora(Fedora::new())),
            "python" => Ok(Self::Python(Python::new(config))),
            "rust" => Ok(Self::Rust(Rust::new())),
            "rustup" => Ok(Self::Rustup(Rustup::new())),
            "void" => Ok(Self::Void(Xbps::new())),
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

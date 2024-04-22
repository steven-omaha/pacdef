pub mod actual;
pub mod backend_trait;
pub mod macros;
mod root;
pub mod todo_per_backend;

use crate::backend::backend_trait::Switches;
use crate::backend::backend_trait::Text;
use crate::Group;
use crate::Package;
use anyhow::Result;
use backend_trait::Backend;
use std::collections::HashSet;
use std::rc::Rc;

use self::actual::{
    fedora::Fedora, flatpak::Flatpak, python::Python, rust::Rust, rustup::Rustup, void::Void,
};

#[derive(Debug)]
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
    pub fn iter() -> impl Iterator<Item = Self> {
        vec![
            #[cfg(feature = "arch")]
            Self::Arch(actual::arch::Arch::new()),
            #[cfg(feature = "debian")]
            Self::Debian(actual::debian::Debian::new()),
            Self::Flatpak(Flatpak::new()),
            Self::Fedora(Fedora::new()),
            Self::Python(Python::new()),
            Self::Rust(Rust::new()),
            Self::Rustup(Rustup::new()),
            Self::Void(Void::new()),
        ]
        .into_iter()
    }
}

mod macros;
mod pacman;
mod rust;

use std::collections::HashSet;
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::Package;

pub use pacman::Pacman;
pub use rust::Rust;
pub(in crate::backend) type Switches = &'static [&'static str];
pub(in crate::backend) type Text = &'static str;

#[derive(Debug)]
pub enum Backends {
    Pacman,
    Rust,
}

impl Backends {
    pub fn iter() -> BackendIter {
        BackendIter(Some(Self::Pacman))
    }

    pub fn get(&self) -> Box<dyn Backend> {
        match self {
            Self::Pacman => Box::new(Pacman {
                packages: HashSet::new(),
            }),
            Self::Rust => Box::new(Rust {
                packages: HashSet::new(),
            }),
        }
    }
}

pub struct BackendIter(Option<Backends>);

impl Iterator for BackendIter {
    type Item = Box<dyn Backend>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            Some(Backends::Pacman) => {
                self.0 = Some(Backends::Rust);
                Some(Box::new(Pacman::new()))
            }
            Some(Backends::Rust) => {
                self.0 = None;
                Some(Box::new(Rust::new()))
            }
            None => None,
        }
    }
}

pub trait Backend {
    fn get_binary(&self) -> Text;
    fn get_section(&self) -> Text;
    fn get_switches_install(&self) -> Switches;
    fn get_switches_remove(&self) -> Switches;

    /// Get all packages that are installed in the system.
    fn get_all_installed_packages(&self) -> HashSet<Package>;

    /// Get all packages that were installed in the system explicitly.
    fn get_explicitly_installed_packages(&self) -> HashSet<Package>;

    /// Install the specified packages.
    fn install_packages(&self, packages: Vec<Package>) {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_install());
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.exec();
    }

    /// Remove the specified packages.
    fn remove_packages(&self, packages: Vec<Package>) {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_remove());
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.exec();
    }

    /// extract packages from its own section as read from group files
    fn extract_packages_from_group_file_content(&self, content: &str) -> HashSet<Package> {
        content
            .lines()
            .skip_while(|line| !line.starts_with(&format!("[{}]", self.get_section())))
            .skip(1)
            .filter(|line| !line.starts_with('['))
            .fuse()
            .map(Package::from)
            .collect()
    }

    fn add_packages(&mut self, packages: HashSet<Package>);
}

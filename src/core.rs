use std::collections::HashSet;
use std::path::PathBuf;
use std::process::exit;

use crate::action;
use crate::cmd::run_install_command;
use crate::db::{get_all_installed_packages, get_explicitly_installed_packages};
use crate::group::{Group, GROUPS_DIR};
use crate::package::Package;

use clap::ArgMatches;

pub struct Pacdef {
    pub(crate) args: ArgMatches,
    pub(crate) groups: Option<HashSet<Group>>,
    // action: Box<dyn Fn(Self)>,
}

impl Pacdef {
    pub fn new(args: ArgMatches, groups: HashSet<Group>) -> Self {
        Self {
            args,
            groups: Some(groups),
            // action: Box::new(Self::install_packages),
        }
    }

    pub(crate) fn take_packages_as_set(&mut self) -> HashSet<Package> {
        self.groups
            .take()
            .unwrap()
            .into_iter()
            .flat_map(|g| g.packages)
            .collect()
    }

    pub(crate) fn get_packages_to_install(&mut self) -> Vec<Package> {
        let managed = self.take_packages_as_set();
        let local_packages = get_all_installed_packages();
        let mut diff: Vec<_> = managed
            .into_iter()
            .filter(|p| !local_packages.contains(p))
            .collect();
        diff.sort_unstable();
        diff
    }

    pub(crate) fn install_packages(mut self) {
        let diff = self.get_packages_to_install();
        if diff.is_empty() {
            println!("nothing to do");
            exit(0);
        }
        println!("Would install the following packages:");
        for p in &diff {
            println!("  {p}");
        }
        println!();
        crate::ui::get_user_confirmation();

        run_install_command(diff);
    }

    pub fn run_action_from_arg(self) {
        match self.args.subcommand() {
            // Some((args::EDIT, groups)) => println!("{groups:#?}"),
            Some((action::EDIT, groups)) => self.edit_group_files(groups),
            Some((action::GROUPS, _)) => self.show_groups(),
            Some((action::SYNC, _)) => self.install_packages(),
            Some((action::UNMANAGED, _)) => self.show_unmanaged_packages(),
            Some((action::VERSION, _)) => self.show_version(),
            _ => todo!(),
        }
    }

    pub(crate) fn edit_group_files(&self, groups: &ArgMatches) {
        let files: Vec<_> = groups
            .get_many::<String>("group")
            .unwrap()
            .map(|file| {
                let mut buf = PathBuf::from(GROUPS_DIR);
                buf.push(file);
                buf
            })
            .collect();
        for f in files {
            println!("{f:#?}");
        }
    }

    pub(crate) fn show_version(self) {
        println!("pacdef, version: {}", env!("CARGO_PKG_VERSION"))
    }

    pub(crate) fn show_unmanaged_packages(mut self) {
        for p in &self.get_unmanaged_packages() {
            println!("{p}");
        }
    }

    /// Returns a `Vec` of alphabetically sorted unmanaged packages.
    pub(crate) fn get_unmanaged_packages(&mut self) -> Vec<Package> {
        let managed = self.take_packages_as_set();
        let explicitly_installed = get_explicitly_installed_packages();
        let mut result: Vec<_> = explicitly_installed
            .into_iter()
            .filter(|p| !managed.contains(p))
            .collect();
        result.sort_unstable();
        result
    }

    pub(crate) fn show_groups(mut self) {
        let groups = self.groups.take().unwrap();
        let mut vec: Vec<_> = groups.iter().collect();
        vec.sort_unstable();
        for g in vec {
            println!("{}", g.name);
        }
    }
}

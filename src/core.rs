use std::collections::HashSet;
use std::process::exit;

use anyhow::{bail, Context, Result};
use clap::ArgMatches;

use crate::action;
use crate::backend::{Backend, Backends, Pacman};
use crate::cmd::run_edit_command;
use crate::ui::get_user_confirmation;
use crate::Group;
use crate::Package;

pub struct Pacdef {
    pub(crate) args: ArgMatches,
    pub(crate) groups: HashSet<Group>,
}

impl Pacdef {
    pub fn new(args: ArgMatches, groups: HashSet<Group>) -> Self {
        Self { args, groups }
    }

    // pub(crate) fn get_packages_to_install(&mut self) -> Vec<Package> {
    //     let managed = self.take_packages_as_set();
    //     let local_packages = Pacman::get_all_installed_packages();
    //     let mut diff: Vec<_> = managed
    //         .into_iter()
    //         .filter(|p| !local_packages.contains(p))
    //         .collect();
    //     diff.sort_unstable();
    //     diff
    // }

    pub(crate) fn install_packages(&self) {
        for b in Backends::iter() {
            println!("{}", b.get_binary());
        }
        // let diff = self.get_packages_to_install();
        // if diff.is_empty() {
        //     println!("nothing to do");
        //     exit(0);
        // }
        // println!("Would install the following packages:");
        // for p in &diff {
        //     println!("  {p}");
        // }
        // crate::ui::get_user_confirmation();

        // Pacman::install_packages(diff);
    }

    #[allow(clippy::unit_arg)]
    pub fn run_action_from_arg(self) -> Result<()> {
        match self.args.subcommand() {
            // Some((action::CLEAN, _)) => Ok(self.clean_packages()),
            Some((action::EDIT, groups)) => self.edit_group_files(groups).context("editing"),
            // Some((action::GROUPS, _)) => Ok(self.show_groups()),
            Some((action::SYNC, _)) => Ok(self.install_packages()),
            // Some((action::UNMANAGED, _)) => Ok(self.show_unmanaged_packages()),
            Some((action::VERSION, _)) => Ok(self.show_version()),
            _ => todo!(),
        }
    }

    pub(crate) fn edit_group_files(&self, groups: &ArgMatches) -> Result<()> {
        let files: Vec<_> = groups
            .get_many::<String>("group")
            .context("getting group from args")?
            .map(|file| {
                let mut buf = crate::path::get_pacdef_group_dir().unwrap();
                buf.push(file);
                buf
            })
            .collect();
        if run_edit_command(&files)
            .context("running editor")?
            .success()
        {
            Ok(())
        } else {
            bail!("editor exited with error")
        }
    }

    pub(crate) fn show_version(self) {
        println!("pacdef, version: {}", env!("CARGO_PKG_VERSION"))
    }

    // pub(crate) fn show_unmanaged_packages(mut self) {
    //     for p in &self.get_unmanaged_packages() {
    //         println!("{p}");
    // }
    // }

    // /// Returns a `Vec` of alphabetically sorted unmanaged packages.
    // pub(crate) fn get_unmanaged_packages(&mut self) -> Vec<Package> {
    //     let managed = self.take_packages_as_set();
    //     let explicitly_installed = Pacman::get_explicitly_installed_packages();
    //     let mut result: Vec<_> = explicitly_installed
    //         .into_iter()
    //         .filter(|p| !managed.contains(p))
    //         .collect();
    //     result.sort_unstable();
    //     result
    // }

    // pub(crate) fn show_groups(mut self) {
    //     let groups = self.groups.take().unwrap();
    //     let mut vec: Vec<_> = groups.iter().collect();
    //     vec.sort_unstable();
    //     for g in vec {
    //         println!("{}", g.name);
    //     }
    // }

    // fn clean_packages(mut self) {
    //     let unmanaged = self.get_unmanaged_packages();
    //     if unmanaged.is_empty() {
    //         println!("nothing to do");
    //         return;
    //     }

    //     println!("Would remove the following packages and their dependencies:");
    //     for p in &unmanaged {
    //         println!("  {p}");
    //     }
    //     get_user_confirmation();
    //     Pacman::remove_packages(unmanaged);
    // }
}

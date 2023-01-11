use std::collections::HashSet;

use anyhow::{ensure, Context, Result};
use clap::ArgMatches;

use crate::action;
use crate::backend::{Backend, Backends, ToDoPerBackend};
use crate::cmd::run_edit_command;
use crate::env::get_single_var;
use crate::ui::get_user_confirmation;
use crate::Group;

pub struct Pacdef {
    args: ArgMatches,
    groups: HashSet<Group>,
}

impl Pacdef {
    #[must_use]
    pub fn new(args: ArgMatches, groups: HashSet<Group>) -> Self {
        Self { args, groups }
    }

    #[allow(clippy::unit_arg)]
    pub fn run_action_from_arg(self) -> Result<()> {
        // TODO import
        // TODO new
        // TODO remove
        // TODO review
        // TODO search
        match self.args.subcommand() {
            Some((action::CLEAN, _)) => Ok(self.clean_packages()),
            Some((action::EDIT, groups)) => {
                self.edit_group_files(groups).context("editing group files")
            }
            Some((action::GROUPS, _)) => Ok(self.show_groups()),
            Some((action::SHOW, groups)) => {
                self.show_group_content(groups).context("showing groups")
            }
            Some((action::SYNC, _)) => Ok(self.install_packages()),
            Some((action::UNMANAGED, _)) => Ok(self.show_unmanaged_packages()),
            Some((action::VERSION, _)) => Ok(self.show_version()),
            Some((_, _)) => todo!(),
            None => {
                unreachable!("argument parser requires some subcommand to return an `ArgMatches`")
            }
        }
    }

    fn get_missing_packages(&self) -> ToDoPerBackend {
        let mut to_install = ToDoPerBackend::new();

        for mut backend in Backends::iter() {
            backend.load(&self.groups);

            match backend.get_missing_packages_sorted() {
                Ok(diff) => to_install.push((backend, diff)),
                Err(error) => show_error(error, backend),
            };
        }

        to_install
    }

    fn install_packages(&self) {
        let to_install = self.get_missing_packages();

        if to_install.nothing_to_do_for_all_backends() {
            println!("nothing to do");
            return;
        }

        to_install.show("install".into());

        if !get_user_confirmation() {
            return;
        };

        to_install.install_missing_packages();
    }

    fn edit_group_files(&self, groups: &ArgMatches) -> Result<()> {
        let group_dir = crate::path::get_pacdef_group_dir()?;

        let files: Vec<_> = groups
            .get_many::<String>("group")
            .context("getting group from args")?
            .map(|file| {
                let mut buf = group_dir.clone();
                buf.push(file);
                buf
            })
            .collect();

        for file in &files {
            ensure!(
                file.exists(),
                "group file {} not found",
                file.to_string_lossy()
            );
        }

        let success = run_edit_command(&files)
            .context("running editor")?
            .success();

        ensure!(success, "editor exited with error");
        Ok(())
    }

    fn show_version(self) {
        println!("pacdef, version: {}", env!("CARGO_PKG_VERSION"));
    }

    fn show_unmanaged_packages(self) {
        let unmanaged_per_backend = &self.get_unmanaged_packages();

        unmanaged_per_backend.show(None);
    }

    fn get_unmanaged_packages(self) -> ToDoPerBackend {
        let mut result = ToDoPerBackend::new();

        for mut backend in Backends::iter() {
            backend.load(&self.groups);

            match backend.get_unmanaged_packages_sorted() {
                Ok(unmanaged) => result.push((backend, unmanaged)),
                Err(error) => show_error(error, backend),
            };
        }
        result
    }

    fn show_groups(self) {
        let mut vec: Vec<_> = self.groups.iter().collect();
        vec.sort_unstable();
        for g in vec {
            println!("{}", g.name);
        }
    }

    fn clean_packages(self) {
        let to_remove = self.get_unmanaged_packages();

        if to_remove.is_empty() {
            println!("nothing to do");
            return;
        }

        to_remove.show("remove".into());

        if !get_user_confirmation() {
            return;
        };

        for (backend, packages) in to_remove.into_iter() {
            backend.remove_packages(packages);
        }
    }

    fn show_group_content(&self, groups: &ArgMatches) -> Result<()> {
        let groups = groups.get_many::<String>("group").unwrap();
        for arg_group in groups {
            let group = self.groups.iter().find(|g| g.name == *arg_group).unwrap();
            println!("{group}");
        }
        Ok(())
    }
}

fn show_error(error: anyhow::Error, backend: Box<dyn Backend>) {
    let section = backend.get_section();
    match get_single_var("RUST_BACKTRACE") {
        Some(s) => {
            if s == "1" || s == "full" {
                println!("WARNING: skipping backend '{section}': {error:?}\n");
            }
        }
        None => println!("WARNING: skipping backend '{section}': {error}"),
    }
}

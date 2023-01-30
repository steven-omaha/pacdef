use std::collections::HashSet;
use std::fs::{remove_file, File};
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use anyhow::{ensure, Context, Result};
use clap::ArgMatches;

use crate::action::*;
use crate::args;
use crate::backend::{Backend, Backends, ToDoPerBackend};
use crate::cmd::run_edit_command;
use crate::env::get_single_var;
use crate::path::get_pacdef_group_dir;
use crate::review;
use crate::search;
use crate::ui::get_user_confirmation;
use crate::Config;
use crate::Group;

pub struct Pacdef {
    args: ArgMatches,
    config: Config,
    groups: HashSet<Group>,
}

// TODO review
impl Pacdef {
    #[must_use]
    pub fn new(args: ArgMatches, config: Config, groups: HashSet<Group>) -> Self {
        Self {
            args,
            config,
            groups,
        }
    }

    #[allow(clippy::unit_arg)]
    pub fn run_action_from_arg(mut self) -> Result<()> {
        match self.args.subcommand() {
            Some((CLEAN, _)) => self.clean_packages(),
            Some((EDIT, args)) => self.edit_group_files(args).context("editing group files"),
            Some((GROUPS, _)) => Ok(self.show_groups()),
            Some((IMPORT, args)) => self.import_groups(args).context("importing groups"),
            Some((NEW, args)) => self.new_groups(args).context("creating new group files"),
            Some((REMOVE, args)) => self.remove_groups(args).context("removing groups"),
            Some((REVIEW, _)) => review::review(self.get_unmanaged_packages(), self.groups)
                .context("removing groups"),
            Some((SHOW, args)) => self.show_group_content(args).context("showing groups"),
            Some((SEARCH, args)) => {
                search::search_packages(args, &self.groups).context("searching packages")
            }
            Some((SYNC, _)) => self.install_packages(),
            Some((UNMANAGED, _)) => Ok(self.show_unmanaged_packages()),
            Some((VERSION, _)) => Ok(self.show_version()),
            Some((_, _)) => todo!(),
            None => {
                unreachable!("argument parser requires some subcommand to return an `ArgMatches`")
            }
        }
    }

    fn get_missing_packages(&mut self) -> ToDoPerBackend {
        let mut to_install = ToDoPerBackend::new();

        for backend in Backends::iter() {
            let mut backend = self.overwrite_values_from_config(backend);

            backend.load(&self.groups);

            match backend.get_missing_packages_sorted() {
                Ok(diff) => to_install.push((backend, diff)),
                Err(error) => show_error(error, backend),
            };
        }

        to_install
    }

    fn overwrite_values_from_config(&mut self, backend: Box<dyn Backend>) -> Box<dyn Backend> {
        if backend.get_section() == "pacman" {
            Box::new(crate::backend::Pacman {
                binary: self.config.aur_helper.clone(),
                aur_rm_args: self.config.aur_rm_args.take(),
                packages: HashSet::new(),
            })
        } else {
            backend
        }
    }

    fn install_packages(&mut self) -> Result<()> {
        let to_install = self.get_missing_packages();

        if to_install.nothing_to_do_for_all_backends() {
            println!("nothing to do");
            return Ok(());
        }

        to_install.show("install".into());

        if !get_user_confirmation() {
            return Ok(());
        };

        to_install.install_missing_packages()
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
        println!("{}", get_version_string());
    }

    fn show_unmanaged_packages(mut self) {
        let unmanaged_per_backend = &self.get_unmanaged_packages();

        unmanaged_per_backend.show(None);
    }

    fn get_unmanaged_packages(&mut self) -> ToDoPerBackend {
        let mut result = ToDoPerBackend::new();

        for backend in Backends::iter() {
            let mut backend = self.overwrite_values_from_config(backend);

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

    fn clean_packages(mut self) -> Result<()> {
        let to_remove = self.get_unmanaged_packages();

        if to_remove.is_empty() {
            println!("nothing to do");
            return Ok(());
        }

        to_remove.show("remove".into());

        if !get_user_confirmation() {
            return Ok(());
        };

        to_remove.remove_unmanaged_packages()
    }

    fn show_group_content(&self, groups: &ArgMatches) -> Result<()> {
        let mut iter = groups.get_many::<String>("group").unwrap().peekable();

        while let Some(arg_group) = iter.next() {
            let group = self.groups.iter().find(|g| g.name == *arg_group).unwrap();
            println!("{group}");
            if iter.peek().is_some() {
                println!();
            }
        }

        Ok(())
    }

    fn import_groups(&self, args: &ArgMatches) -> Result<()> {
        let files = args::get_absolutized_file_paths(args);
        let groups_dir = get_pacdef_group_dir()?;

        for target in files {
            let target_name = target.file_name().unwrap().to_str().unwrap();

            if !target.exists() {
                println!("file {target_name} does not exist, skipping");
                continue;
            }

            let mut link = groups_dir.clone();
            link.push(target_name);

            if link.exists() {
                println!("group {target_name} already exists, skipping");
            } else {
                symlink(target, link)?;
            }
        }

        Ok(())
    }

    fn remove_groups(&self, arg_match: &ArgMatches) -> Result<()> {
        let paths = get_assumed_group_file_names(arg_match)?;

        for file in &paths {
            ensure!(file.exists(), "did not find the group under {file:?}");
        }

        for file in paths {
            remove_file(file)?;
        }

        Ok(())
    }

    fn new_groups(&self, arg: &ArgMatches) -> Result<()> {
        let paths = get_assumed_group_file_names(arg)?;

        for file in &paths {
            ensure!(!file.exists(), "group already exists under {file:?}");
        }

        for file in &paths {
            File::create(file)?;
        }

        if arg.get_flag("edit") {
            let success = run_edit_command(&paths)
                .context("running editor")?
                .success();

            ensure!(success, "editor exited with error");
        }

        Ok(())
    }
}

fn get_assumed_group_file_names(arg_match: &ArgMatches) -> Result<Vec<PathBuf>> {
    let groups_dir = get_pacdef_group_dir()?;

    let paths: Vec<_> = arg_match
        .get_many::<String>("groups")
        .unwrap()
        .map(|s| {
            let mut possible_group_file = groups_dir.clone();
            possible_group_file.push(s);
            possible_group_file
        })
        .collect();

    Ok(paths)
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

pub(crate) const fn get_version_string() -> &'static str {
    concat!(
        "pacdef, version: ",
        env!("CARGO_PKG_VERSION"),
        " (",
        env!("GIT_HASH"),
        ")",
    )
}

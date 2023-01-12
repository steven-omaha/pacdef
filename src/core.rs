use std::collections::HashSet;
use std::fs::{remove_file, File};
use std::iter::Peekable;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::vec::IntoIter;

use anyhow::{ensure, Context, Result};
use clap::ArgMatches;
use regex::Regex;

use crate::action;
use crate::args;
use crate::backend::{Backend, Backends, ToDoPerBackend};
use crate::cmd::run_edit_command;
use crate::env::get_single_var;
use crate::grouping::{Package, Section};
use crate::path::get_pacdef_group_dir;
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
        // TODO review
        match self.args.subcommand() {
            Some((action::CLEAN, _)) => Ok(self.clean_packages()),
            Some((action::EDIT, groups)) => {
                self.edit_group_files(groups).context("editing group files")
            }
            Some((action::GROUPS, _)) => Ok(self.show_groups()),
            Some((action::IMPORT, files)) => self.import_groups(files).context("importing groups"),
            Some((action::NEW, files)) => {
                self.new_groups(files).context("creating new group files")
            }
            Some((action::REMOVE, groups)) => self.remove_groups(groups).context("removing groups"),
            Some((action::SHOW, groups)) => {
                self.show_group_content(groups).context("showing groups")
            }
            Some((action::SEARCH, args)) => {
                self.search_packages(args).context("searching packages")
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

        for file in paths.iter() {
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

    fn search_packages(&self, args: &ArgMatches) -> Result<()> {
        let search_string = args
            .get_one::<String>("string")
            .context("getting search string from arg")?;

        let re = Regex::new(search_string)?;

        let mut vec = vec![];

        for group in &self.groups {
            for section in &group.sections {
                for package in &section.packages {
                    if re.is_match(&package.name) {
                        vec.push((group, section, package))
                    }
                }
            }
        }

        print_triples(vec);

        Ok(())
    }
}

fn print_triples(mut vec: Vec<(&Group, &Section, &Package)>) {
    vec.sort_unstable();

    let mut g0 = String::new();
    let mut s0 = String::new();

    let mut iter = vec.into_iter().peekable();

    while let Some((g, s, p)) = iter.next() {
        print_group_if_changed(g, &g0, &mut s0);
        print_section_if_changed(s, &s0);
        println!("{p}");
        save_group_and_section_name(&mut g0, g, &mut s0, s);
        print_separator_unless_exhausted(&mut iter, &g0);
    }
}

fn save_group_and_section_name(g0: &mut String, g: &Group, s0: &mut String, s: &Section) {
    *g0 = g.name.clone();
    *s0 = s.name.clone();
}

fn print_separator_unless_exhausted(
    iter: &mut Peekable<IntoIter<(&Group, &Section, &Package)>>,
    g0: &String,
) {
    if let Some((g, _, _)) = iter.peek() {
        if g.name != *g0 {
            println!();
        }
    }
}

fn print_section_if_changed(s: &Section, s0: &String) {
    if s.name != *s0 {
        println!("[{}]", s.name);
    }
}

fn print_group_if_changed(g: &Group, g0: &String, s0: &mut String) {
    if g.name != *g0 {
        println!("{}", g.name);
        for _ in 0..g.name.len() {
            print!("-");
        }
        println!();
        s0.clear();
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

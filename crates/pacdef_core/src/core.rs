use std::collections::{HashMap, HashSet};
use std::fs::{remove_file, File};
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::ArgMatches;
use const_format::formatcp;

use crate::action::*;
use crate::args;
use crate::backend::{Backend, Backends, ToDoPerBackend};
use crate::cmd::run_edit_command;
use crate::env::get_single_var;
use crate::path::get_group_dir;
use crate::review;
use crate::search;
use crate::ui::get_user_confirmation;
use crate::Config;
use crate::Group;

const UNREACHABLE_ARM: &str = "argument parser requires some subcommand to return an `ArgMatches`";
const ACTION_NOT_MATCHED: &str = "could not match action";

/// Most data that is required during runtime of the program.
pub struct Pacdef {
    args: ArgMatches,
    config: Config,
    groups: HashSet<Group>,
}

impl Pacdef {
    /// Creates a new [`Pacdef`]. `config` should be passed from [`Config::load`], and `args` from
    /// [`args::get`].
    #[must_use]
    pub const fn new(args: ArgMatches, config: Config, groups: HashSet<Group>) -> Self {
        Self {
            args,
            config,
            groups,
        }
    }

    /// Run the action that was provided by the user as first argument.
    ///
    /// For convenience sake, all called functions take a `&self` argument, even if these are not
    /// strictly required.
    ///
    /// # Panics
    ///
    /// Panics if the user passed an unexpected action. This means all fields from `crate::action::Action` must be matched in this function.
    ///
    /// # Errors
    ///
    /// This function propagates errors from the underlying functions.
    #[allow(clippy::unit_arg)]
    pub fn run_action_from_arg(mut self) -> Result<()> {
        match self.args.subcommand() {
            Some(("group", args)) => match args.subcommand() {
                Some((EDIT, args)) => self.edit_group_files(args).context("editing group files"),
                Some((IMPORT, args)) => self.import_groups(args).context("importing groups"),
                Some((LIST, _)) => Ok(self.show_groups()),
                Some((NEW, args)) => self.new_groups(args).context("creating new group files"),
                Some((REMOVE, args)) => self.remove_groups(args).context("removing groups"),
                Some((SHOW, args)) => self.show_group_content(args).context("showing groups"),

                Some((_, _)) => panic!("{ACTION_NOT_MATCHED}"),
                None => unreachable!("{UNREACHABLE_ARM}"),
            },

            Some(("package", args)) => match args.subcommand() {
                Some((CLEAN, _)) => self.clean_packages(),
                Some((REVIEW, _)) => review::review(self.get_unmanaged_packages(), self.groups)
                    .context("review unmanaged packages"),
                Some((SEARCH, args)) => {
                    search::search_packages(args, &self.groups).context("searching packages")
                }
                Some((SYNC, _)) => self.install_packages(),
                Some((UNMANAGED, _)) => self.show_unmanaged_packages(),

                Some((_, _)) => panic!("{ACTION_NOT_MATCHED}"),
                None => unreachable!("{UNREACHABLE_ARM}"),
            },

            Some((VERSION, _)) => Ok(self.show_version()),

            Some((_, _)) => panic!("{ACTION_NOT_MATCHED}"),
            None => unreachable!("{UNREACHABLE_ARM}"),
        }
    }

    fn get_missing_packages(&mut self) -> ToDoPerBackend {
        let mut to_install = ToDoPerBackend::new();

        for backend in Backends::iter() {
            let mut backend = self.overwrite_values_from_config(backend);

            backend.load(&self.groups);

            match backend.get_missing_packages_sorted() {
                Ok(diff) => to_install.push((backend, diff)),
                Err(error) => show_error(&error, &*backend),
            };
        }

        to_install
    }

    fn overwrite_values_from_config(&mut self, backend: Box<dyn Backend>) -> Box<dyn Backend> {
        if backend.get_section() == "arch" {
            Box::new(crate::backend::Arch {
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

        println!("Would install the following packages:\n");
        to_install.show().context("printing things to do")?;

        println!();
        if !get_user_confirmation()? {
            return Ok(());
        };

        to_install.install_missing_packages()
    }

    fn edit_group_files(&self, arg_matches: &ArgMatches) -> Result<()> {
        let group_files = get_group_file_paths_matching_args(arg_matches, &self.groups)
            .context("getting group files for args")?;

        let success = run_edit_command(&group_files)
            .context("running editor")?
            .success();

        ensure!(success, "editor exited with error");
        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn show_version(self) {
        println!("{}", get_version_string());
    }

    fn show_unmanaged_packages(mut self) -> Result<()> {
        let unmanaged_per_backend = &self.get_unmanaged_packages();

        if unmanaged_per_backend.nothing_to_do_for_all_backends() {
            return Ok(());
        }

        unmanaged_per_backend
            .show()
            .context("printing things to do")
    }

    fn get_unmanaged_packages(&mut self) -> ToDoPerBackend {
        let mut result = ToDoPerBackend::new();

        for backend in Backends::iter() {
            let mut backend = self.overwrite_values_from_config(backend);

            backend.load(&self.groups);

            match backend.get_unmanaged_packages_sorted() {
                Ok(unmanaged) => result.push((backend, unmanaged)),
                Err(error) => show_error(&error, &*backend),
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

        if to_remove.nothing_to_do_for_all_backends() {
            println!("nothing to do");
            return Ok(());
        }

        println!("Would remove the following packages:\n");
        to_remove.show().context("printing things to do")?;

        println!();
        if !get_user_confirmation()? {
            return Ok(());
        };

        to_remove.remove_unmanaged_packages()
    }

    fn show_group_content(&self, groups: &ArgMatches) -> Result<()> {
        let mut iter = groups
            .get_many::<String>("group")
            .context("getting groups from args")?
            .peekable();

        let show_more_than_one_group = iter.size_hint().0 > 1;

        while let Some(arg_group) = iter.next() {
            let group = self
                .groups
                .iter()
                .find(|g| g.name == *arg_group)
                .ok_or_else(|| anyhow!("group {} not found", *arg_group))?;

            if show_more_than_one_group {
                let name = &group.name;
                println!("{name}");
                for _ in 0..name.len() {
                    print!("-");
                }
                println!();
            }

            println!("{group}");
            if iter.peek().is_some() {
                println!();
            }
        }

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn import_groups(&self, args: &ArgMatches) -> Result<()> {
        let files = args::get_absolutized_file_paths(args)?;
        let groups_dir = get_group_dir()?;

        for target in files {
            let target_name = target
                .file_name()
                .context("path should not end in '..'")?
                .to_str()
                .context("filename is not valid UTF-8")?;

            if !target.exists() {
                eprintln!("file {target_name} does not exist, skipping");
                continue;
            }

            let mut link = groups_dir.clone();
            link.push(target_name);

            if link.exists() {
                eprintln!("group {target_name} already exists, skipping");
            } else {
                symlink(target, link)?;
            }
        }

        Ok(())
    }

    fn remove_groups(&self, arg_match: &ArgMatches) -> Result<()> {
        let paths = get_group_file_paths_matching_args(arg_match, &self.groups)?;

        for file in paths {
            remove_file(file)?;
        }

        Ok(())
    }

    fn new_groups(&self, arg_matches: &ArgMatches) -> Result<()> {
        let paths = get_group_file_paths_matching_args(arg_matches, &self.groups)?;

        for file in &paths {
            ensure!(!file.exists(), "group already exists under {file:?}");
        }

        for file in &paths {
            File::create(file)?;
        }

        if arg_matches.get_flag("edit") {
            let success = run_edit_command(&paths)
                .context("running editor")?
                .success();

            ensure!(success, "editor exited with error");
        }

        Ok(())
    }
}

/// For the provided CLI arguments, get the path to each corresponding group file.
///
/// # Errors
///
/// This function will return an error if any of the arguments do not match one of group names.
fn get_group_file_paths_matching_args<'a>(
    arg_match: &ArgMatches,
    groups: &'a HashSet<Group>,
) -> Result<Vec<&'a Path>> {
    let file_names: Vec<_> = arg_match
        .get_many::<String>("groups")
        .context("getting groups from args")
        .map_err(|_| crate::errors::Error::NoGroupFilesInArguments)?
        .collect();

    let name_group_map: HashMap<&str, &Group> =
        groups.iter().map(|g| (g.name.as_str(), g)).collect();

    let mut result = Vec::new();

    for file in file_names {
        match name_group_map.get(file.as_str()) {
            Some(group) => {
                result.push(group.path.as_path());
            }
            None => bail!("group file {} not found", file),
        }
    }

    Ok(result)
}

#[allow(clippy::option_if_let_else)]
fn show_error(error: &anyhow::Error, backend: &dyn Backend) {
    let section = backend.get_section();
    match get_single_var("RUST_BACKTRACE") {
        Some(s) => {
            if s == "1" || s == "full" {
                eprintln!("WARNING: skipping backend '{section}':");
                for err in error.chain() {
                    eprintln!("  {err}");
                }
            }
        }
        None => eprintln!("WARNING: skipping backend '{section}': {error}"),
    }
}

pub const fn get_version_string() -> &'static str {
    const VERSION: &str = concat!("pacdef, version: ", env!("CARGO_PKG_VERSION"));
    const HASH: &str = env!("GIT_HASH");

    if HASH.is_empty() {
        VERSION
    } else {
        formatcp!("{VERSION} ({HASH})")
    }
}

use std::collections::{HashMap, HashSet};
use std::fs::{remove_file, File};
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::{bail, ensure, Context, Result};
use clap::ArgMatches;
use const_format::formatcp;

use crate::action::*;
use crate::args::{self, Arguments, GroupArguments, PackageArguments};
use crate::backend::{Backend, Backends, ToDoPerBackend};
use crate::cmd::run_edit_command;
use crate::env::get_single_var;
use crate::path::{binary_in_path, get_group_dir};
use crate::review;
use crate::search;
use crate::ui::get_user_confirmation;
use crate::Config;
use crate::Group;

const UNREACHABLE_ARM: &str = "argument parser requires some subcommand to return an `ArgMatches`";
const ACTION_NOT_MATCHED: &str = "could not match action";

/// Most data that is required during runtime of the program.
#[allow(dead_code)] // "`config` is only needed on Arch Linux"
pub struct Pacdef {
    args: Option<Arguments>,
    config: Config,
    groups: HashSet<Group>,
}

impl Pacdef {
    /// Creates a new [`Pacdef`]. `config` should be passed from [`Config::load`], and `args` from
    /// [`args::get`].
    #[must_use]
    pub const fn new(args: Arguments, config: Config, groups: HashSet<Group>) -> Self {
        Self {
            args: Some(args),
            config,
            groups,
        }
    }

    /// Run the action that was provided by the user as first argument.
    ///
    /// For convenience sake, all called functions take a `&self` argument, even if
    /// these are not strictly required.
    ///
    /// # Panics
    ///
    /// Panics if the user passed an unexpected action. This means all fields from
    /// `crate::action::Action` must be matched in this function.
    ///
    /// # Errors
    ///
    /// This function propagates errors from the underlying functions.
    #[allow(clippy::unit_arg)]
    pub fn run_action_from_arg(mut self) -> Result<()> {
        match self.args.take().unwrap() {
            Arguments::Group(group_args) => self.run_group_subcommand(&group_args),
            Arguments::Package(package_args) => self.run_package_subcommand(&package_args),
            Arguments::Version => Ok(self.show_version()),
            // Some(("group", args)) => match args.subcommand() {
            //     Some((EDIT, args)) => self.edit_group_files(args).context("editing group files"),
            //     Some((IMPORT, args)) => self.import_groups(args).context("importing groups"),
            //     Some((LIST, _)) => Ok(self.show_groups()),
            //     Some((NEW, args)) => self.new_groups(args).context("creating new group files"),
            //     Some((REMOVE, args)) => self.remove_groups(args).context("removing groups"),
            //     Some((SHOW, args)) => self.show_group_content(args).context("showing groups"),

            //     Some((_, _)) => panic!("{ACTION_NOT_MATCHED}"),
            //     None => unreachable!("{UNREACHABLE_ARM}"),

            // Some(("package", args)) => match args.subcommand() {
            //     Some((CLEAN, args)) => self.clean_packages(&args.clone()),
            //     Some((REVIEW, _)) => review::review(self.get_unmanaged_packages()?, self.groups)
            //         .context("review unmanaged packages"),
            //     Some((SEARCH, args)) => {
            //         search::search_packages(args, &self.groups).context("searching packages")
            //     }
            //     Some((SYNC, args)) => self.install_packages(&args.clone()), // TODO fix cloning
            //     Some((UNMANAGED, _)) => self.show_unmanaged_packages(),

            //     Some((_, _)) => panic!("{ACTION_NOT_MATCHED}"),
            //     None => unreachable!("{UNREACHABLE_ARM}"),
            // },

            // Some((VERSION, _)) => Ok(self.show_version()),

            // Some((_, _)) => panic!("{ACTION_NOT_MATCHED}"),
            // None => unreachable!("{UNREACHABLE_ARM}"),
        }
    }

    fn run_group_subcommand(self, args: &GroupArguments) -> Result<()> {
        match args.action {
            GroupAction::Edit => self.edit_group_files(&args.groups),
            GroupAction::Import => self.import_groups(&args.groups),
            GroupAction::List => Ok(self.show_groups()),
            GroupAction::New => self.new_groups(&args.groups, args.edit),
            GroupAction::Remove => self.remove_groups(&args.groups),
            GroupAction::Show => Ok(self.show_groups()),
        }
    }

    fn run_package_subcommand(mut self, args: &PackageArguments) -> Result<()> {
        match args.action {
            PackageAction::Clean => self.clean_packages(args.noconfirm),
            PackageAction::Review => review::review(self.get_unmanaged_packages()?, self.groups),
            PackageAction::Search => search::search_packages(&args.regex, &self.groups),
            PackageAction::Sync => self.install_packages(args.noconfirm),
            PackageAction::Unmanaged => self.show_unmanaged_packages(),
        }
    }

    fn get_missing_packages(&mut self) -> Result<ToDoPerBackend> {
        let mut to_install = ToDoPerBackend::new();

        for mut backend in Backends::iter() {
            if self
                .config
                .disabled_backends
                .contains(&backend.get_section().to_string())
            {
                continue;
            }

            if !binary_in_path(backend.get_binary())? {
                continue;
            }

            self.overwrite_values_from_config(&mut *backend);
            backend.load(&self.groups);

            match backend.get_missing_packages_sorted() {
                Ok(diff) => to_install.push((backend, diff)),
                Err(error) => show_error(&error, &*backend),
            };
        }

        Ok(to_install)
    }

    #[allow(clippy::unused_self, unused_variables)]
    fn overwrite_values_from_config(&mut self, backend: &mut dyn Backend) {
        #[cfg(feature = "arch")]
        {
            if let Some(arch) = backend.as_any_mut().downcast_mut::<crate::backend::Arch>() {
                arch.binary = self.config.aur_helper.clone();
                arch.aur_rm_args = self.config.aur_rm_args.clone();
            }
        }

        if let Some(flatpak) = backend
            .as_any_mut()
            .downcast_mut::<crate::backend::Flatpak>()
        {
            flatpak.systemwide = self.config.flatpak_systemwide;
        }
    }

    fn install_packages(&mut self, noconfirm: bool) -> Result<()> {
        let to_install = self.get_missing_packages()?;

        if to_install.nothing_to_do_for_all_backends() {
            println!("nothing to do");
            return Ok(());
        }

        println!("Would install the following packages:\n");
        to_install.show().context("printing things to do")?;

        println!();
        if noconfirm {
            println!("proceeding without confirmation");
        } else if !get_user_confirmation()? {
            return Ok(());
        }

        to_install.install_missing_packages(noconfirm)
    }

    fn edit_group_files(&self, groups: &[String]) -> Result<()> {
        let group_files = get_group_file_paths_matching_args(&groups, &self.groups)
            .context("getting group files for args")?;

        let success = run_edit_command(&group_files)
            .context("running editor")?
            .success();

        ensure!(success, "editor exited with error");
        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn show_version(self) {
        println!("{}", get_name_and_version());
    }

    fn show_unmanaged_packages(mut self) -> Result<()> {
        let unmanaged_per_backend = &self.get_unmanaged_packages()?;

        if unmanaged_per_backend.nothing_to_do_for_all_backends() {
            return Ok(());
        }

        unmanaged_per_backend
            .show()
            .context("printing things to do")
    }

    /// Get a list of unmanaged packages per backend.
    ///
    /// This method loops through all enabled `Backend`s whose binary is in `PATH`.
    ///
    /// # Errors
    ///
    /// This function will propagate errors.
    fn get_unmanaged_packages(&mut self) -> Result<ToDoPerBackend> {
        let mut result = ToDoPerBackend::new();

        for mut backend in Backends::iter() {
            if self
                .config
                .disabled_backends
                .contains(&backend.get_section().to_string())
            {
                continue;
            }

            if !binary_in_path(backend.get_binary())? {
                continue;
            }

            self.overwrite_values_from_config(&mut *backend);
            backend.load(&self.groups);

            match backend.get_unmanaged_packages_sorted() {
                Ok(unmanaged) => result.push((backend, unmanaged)),
                Err(error) => show_error(&error, &*backend),
            };
        }
        Ok(result)
    }

    fn show_groups(self) {
        let mut vec: Vec<_> = self.groups.iter().collect();
        vec.sort_unstable();
        for g in vec {
            println!("{}", g.name);
        }
    }

    fn clean_packages(&mut self, noconfirm: bool) -> Result<()> {
        let to_remove = self.get_unmanaged_packages()?;

        if to_remove.nothing_to_do_for_all_backends() {
            println!("nothing to do");
            return Ok(());
        }

        println!("Would remove the following packages:\n");
        to_remove.show().context("printing things to do")?;

        println!();
        if noconfirm {
            println!("proceeding without confirmation");
        } else if !get_user_confirmation()? {
            return Ok(());
        }

        to_remove.remove_unmanaged_packages(noconfirm)
    }

    fn show_group_content(&self, groups: &ArgMatches) -> Result<()> {
        let args: Vec<_> = groups
            .get_many::<String>("groups")
            .context("getting groups from args")?
            .collect();

        let mut errors = vec![];
        let mut groups = vec![];

        // make sure all args exist before doing anything
        for arg_group in &args {
            let group = self.groups.iter().find(|g| g.name == **arg_group);

            let group = match group {
                Some(g) => g,
                None => {
                    errors.push((*arg_group).clone());
                    continue;
                }
            };

            groups.push(group);
        }

        // return an error if any arg was not found
        ensure!(
            errors.is_empty(),
            crate::Error::MultipleGroupsNotFound(errors)
        );

        let show_more_than_one_group = args.len() > 1;

        let mut iter = groups.into_iter().peekable();

        while let Some(group) = iter.next() {
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
    fn import_groups(&self, file_names: &[String]) -> Result<()> {
        let files = args::get_absolutized_file_paths(file_names)?;
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

    fn remove_groups(&self, groups: &[String]) -> Result<()> {
        let paths = get_group_file_paths_matching_args(groups, &self.groups)?;

        for file in paths {
            remove_file(file)?;
        }

        Ok(())
    }

    #[allow(clippy::unused_self)]
    fn new_groups(&self, new_group_names: &[String], edit: bool) -> Result<()> {
        let group_path = get_group_dir()?;

        // prevent group names that resolve to directories
        for name in new_group_names {
            ensure!(
                *name != ".",
                crate::Error::InvalidGroupName(".".to_string())
            );
            ensure!(
                *name != "..",
                crate::Error::InvalidGroupName("..".to_string())
            );
        }

        let paths: Vec<_> = new_group_names
            .into_iter()
            .map(|name| {
                let mut base = group_path.clone();
                base.push(name);
                base
            })
            .collect();

        for file in &paths {
            ensure!(
                !file.exists(),
                crate::Error::GroupAlreadyExists(file.clone())
            );
        }

        for file in &paths {
            File::create(file)?;
        }

        if edit {
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
    file_names: &[String],
    groups: &'a HashSet<Group>,
) -> Result<Vec<&'a Path>> {
    let name_group_map: HashMap<&str, &Group> =
        groups.iter().map(|g| (g.name.as_str(), g)).collect();

    let mut result = Vec::new();

    for file in file_names {
        match name_group_map.get(file.as_str()) {
            Some(group) => {
                result.push(group.path.as_path());
            }
            None => bail!(crate::errors::Error::GroupFileNotFound(file.clone())),
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

/// If the crate was compiled from git, return `pacdef, <version> (<hash>)`.
/// Otherwise return `pacdef, <version>`.
fn get_name_and_version() -> String {
    let backends = get_included_backends();
    let mut result = format!("pacdef, version: {}\n", get_version_string());
    result.push_str("supported backends:");
    for b in backends {
        result.push_str("\n  ");
        result.push_str(b);
    }

    result
}

/// If the crate was compiled from git, return `<version> (<hash>)`. Otherwise
/// return `<version>`.
pub const fn get_version_string() -> &'static str {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const HASH: &str = env!("GIT_HASH");

    if HASH.is_empty() {
        VERSION
    } else {
        formatcp!("{VERSION} ({HASH})")
    }
}

fn get_included_backends() -> Vec<&'static str> {
    let mut result = vec![];
    for backend in Backends::iter() {
        result.push(backend.get_section());
    }
    result.sort_unstable();
    result
}

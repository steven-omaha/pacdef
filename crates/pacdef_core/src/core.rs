use std::collections::{HashMap, HashSet};
use std::env::current_dir;
use std::fs::{copy, create_dir_all, remove_file, rename, File};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::{bail, ensure, Context, Result};
use const_format::formatcp;

use crate::args::{self, PackageAction};
use crate::backend::{Backend, Backends, ToDoPerBackend};
use crate::cmd::run_edit_command;
use crate::env::get_single_var;
use crate::path::{binary_in_path, get_absolutized_file_paths, get_group_dir};
use crate::search;
use crate::ui::get_user_confirmation;
use crate::Config;
use crate::Group;
use crate::{review, Error};

/// Most data that is required during runtime of the program.
pub struct Pacdef {
    /// The command line arguments. Is an `Option` so that we can take ownership later without cloning.
    args: Option<args::Arguments>,
    /// The config of the program.
    config: Config,
    /// The hashset of all groups.
    groups: HashSet<Group>,
}

impl Pacdef {
    /// Creates a new [`Pacdef`]. `config` should be passed from [`Config::load`], and `args` from
    /// [`args::get`].
    #[must_use]
    pub const fn new(args: args::Arguments, config: Config, groups: HashSet<Group>) -> Self {
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
    /// # Errors
    ///
    /// This function propagates errors from the underlying functions.
    ///
    /// # Panics
    ///
    /// This function panics if the `args` field is `None`.
    #[allow(clippy::unit_arg)]
    pub fn run_action_from_arg(mut self) -> Result<()> {
        match self
            .args
            .take()
            .expect("if there were no args we would not get to here")
        {
            args::Arguments::Group(group_args) => self.run_group_subcommand(&group_args),
            args::Arguments::Package(package_args) => self.run_package_subcommand(&package_args),
            args::Arguments::Version => Ok(self.show_version()),
        }
    }

    fn run_group_subcommand(self, args: &args::GroupAction) -> Result<()> {
        use args::GroupAction::*;

        match args {
            Edit(args::Groups(groups)) => self.edit_groups(groups),
            Export(args::Groups(groups), args::OutputDir(dir), args::Force(force)) => {
                self.export_groups(groups, dir.as_ref(), *force)
            }
            Import(args::Groups(groups)) => self.import_groups(groups),
            List => self.show_groups(),
            New(args::Groups(groups), args::Edit(edit)) => self.new_groups(groups, *edit),
            Remove(args::Groups(groups)) => self.remove_groups(groups),
            Show(args::Groups(groups)) => self.show_group_content(groups),
        }
    }

    fn run_package_subcommand(mut self, args: &PackageAction) -> Result<()> {
        use args::PackageAction::*;

        match args {
            Clean(args::Noconfirm(noconfirm)) => self.clean_packages(*noconfirm),
            Review => review::review(self.get_unmanaged_packages()?, self.groups),
            Search(args::Regex(regex)) => search::search_packages(regex, &self.groups),
            Sync(args::Noconfirm(noconfirm)) => self.install_packages(*noconfirm),
            Unmanaged => self.show_unmanaged_packages(),
        }
    }

    fn get_missing_packages(&mut self) -> Result<ToDoPerBackend> {
        let mut to_install = ToDoPerBackend::new();

        if self.groups.is_empty() {
            eprintln!("WARNING: no group files found");
        }

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
                Err(error) => show_backend_query_error(&error, &*backend),
            };
        }

        Ok(to_install)
    }

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

    fn edit_groups(&self, groups: &[String]) -> Result<()> {
        if self.groups.is_empty() {
            eprintln!("WARNING: no group files found");
        }

        let group_files: Vec<_> = find_groups_by_name(groups, &self.groups)
            .context("getting group files for args")?
            .into_iter()
            .map(|g| g.path.as_path())
            .collect();

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
    /// This function will propagate errors from the individual backends.
    fn get_unmanaged_packages(&mut self) -> Result<ToDoPerBackend> {
        if self.groups.is_empty() {
            eprintln!("WARNING: no group files found");
        }

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
                Err(error) => show_backend_query_error(&error, &*backend),
            };
        }
        Ok(result)
    }

    /// Print the alphabetically sorted names of all groups to stdout.
    ///
    /// This methods cannot return an error. It returns a `Result` to be consistent
    /// with other methods.
    #[allow(clippy::unnecessary_wraps)]
    fn show_groups(self) -> Result<()> {
        if self.groups.is_empty() {
            eprintln!("WARNING: no group files found");
        }

        let mut vec: Vec<_> = self.groups.iter().collect();
        vec.sort_unstable();
        for g in vec {
            println!("{}", g.name);
        }

        Ok(())
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

    fn show_group_content(&self, args: &[String]) -> Result<()> {
        if self.groups.is_empty() {
            eprintln!("WARNING: no group files found");
        }

        let mut errors = vec![];
        let mut groups = vec![];

        // make sure all args exist before doing anything
        for arg_group in args {
            let possible_group = self.groups.iter().find(|g| g.name == **arg_group);

            let Some(group) = possible_group else {
                errors.push((*arg_group).clone());
                continue;
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
        let files = get_absolutized_file_paths(file_names)?;
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
        if self.groups.is_empty() {
            eprintln!("WARNING: no group files found");
        }

        let found = find_groups_by_name(groups, &self.groups)?;

        for group in found {
            remove_file(&group.path)?;
        }

        Ok(())
    }

    /// Create empty group files.
    ///
    /// If `edit` is `true`, the editor will be run to edit the files after they are
    /// created.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// - a group name is `.` or `..`,
    /// - a group with the same name already exists,
    /// - the editor cannot be run, or
    /// - if we do not have permission to write to the group dir.
    #[allow(clippy::unused_self)]
    fn new_groups(&self, new_groups: &[String], edit: bool) -> Result<()> {
        let group_path = get_group_dir()?;

        // prevent group names that resolve to directories
        for name in new_groups {
            ensure!(
                *name != "." && *name != "..",
                crate::Error::InvalidGroupName(name.clone())
            );
        }

        let paths: Vec<_> = new_groups
            .iter()
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

    /// Export pacdef groups by moving a group file to an output dir. The path of the
    /// group file relative to the group base dir will be replicated under the output
    /// directory.
    ///
    /// By default, the output dir is the current working directory. `output_dir` may be
    /// specified to the path of another directory, in which case `output_dir` must
    /// exist.
    ///
    /// If `force` is `true`, the output file will be overwritten if it exists.
    ///
    /// # Errors
    ///
    /// This function will return an error if
    /// - the group file is a symlink (in which case exporting makes no sense),
    /// - the output file exists and `force` is not `true`, or
    /// - the user does not have permission to write to the output dir.
    ///
    /// # Limitations
    ///
    /// At the moment we cannot export nested group dirs. The user would have to
    /// export every group file individually, or use a shell glob.
    fn export_groups(
        &self,
        names: &[String],
        output_dir: Option<&String>,
        force: bool,
    ) -> Result<()> {
        let groups = find_groups_by_name(names, &self.groups)?;
        let output_dir = match output_dir.map(PathBuf::from) {
            Some(p) => p,
            None => current_dir().context("no output dir specified, getting current directory")?,
        };

        ensure!(
            output_dir.exists() && output_dir.is_dir(),
            "output must be a directory and exist"
        );

        for group in &groups {
            ensure!(!&group.path.is_symlink(), "cannot export symlinks");

            let mut exported_path = output_dir.clone();
            exported_path.push(PathBuf::from(&group.name));

            ensure!(
                !force && !exported_path.exists(),
                "{exported_path:?} already exists"
            );

            create_parent(&exported_path)
                .with_context(|| format!("creating parent dir of {exported_path:?}"))?;
            move_file(&group.path, &exported_path).context("moving file")?;
            symlink(&exported_path, &group.path).context("creating symlink to exported file")?;
        }

        Ok(())
    }
}

/// Create the parent directory of the `path` if that directory does not exist.
///
/// Do nothing otherwise.
///
/// # Panics
///
/// Panics if the path does not have a parent.
///
/// # Errors
///
/// This function will propagate errors from [`std::fs::create_dir_all`].
fn create_parent(path: &Path) -> Result<()> {
    let parent = &path.parent().expect("this should never be /");
    if !parent.is_dir() {
        create_dir_all(parent).context("creating parent dir")?;
    }
    Ok(())
}

/// Move a file from one place to another.
///
/// At first [`std::fs::rename`] is used, which fails if `from` and `to` reside under
/// different filesystems. In case that happens, we will resort to copying the files
/// and then removing `from`.
///
/// # Errors
///
/// This function will return an error if we lack permission to write the file.
fn move_file<P, Q>(from: P, to: Q) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let from = from.as_ref();
    let to = to.as_ref();
    match rename(from, to) {
        Ok(_) => (),
        Err(e) => {
            // CrossesDevices is nightly. See rust #86442.
            // We cannot check that here, so we just assume that
            // that would be the error if permisisons are okay.
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                bail!(e);
            }
            copy(from, to).with_context(|| format!("copying {from:?} to {to:?}"))?;
            remove_file(from).with_context(|| format!("deleting {from:?}"))?;
        }
    };
    Ok(())
}

/// For the provided names, get the group with the same name.
///
/// # Errors
///
/// This function will return an error if any of the file names do not match one
/// of group names.
fn find_groups_by_name<'a>(names: &[String], groups: &'a HashSet<Group>) -> Result<Vec<&'a Group>> {
    let name_group_map: HashMap<&str, &Group> =
        groups.iter().map(|g| (g.name.as_str(), g)).collect();

    let mut result = Vec::new();

    for file in names {
        match name_group_map.get(file.as_str()) {
            Some(group) => {
                result.push(*group);
            }
            None => bail!(Error::GroupFileNotFound(file.clone())),
        }
    }

    Ok(result)
}

/// Show the error chain for an error that has occured when a backend was queried
/// if the `RUST_BACKTRACE` env variable is set to `1` or `full`.
#[allow(clippy::option_if_let_else)]
fn show_backend_query_error(error: &anyhow::Error, backend: &dyn Backend) {
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

/// Get a vector with the names of all backends, sorted alphabetically.
fn get_included_backends() -> Vec<&'static str> {
    let mut result = vec![];
    for backend in Backends::iter() {
        result.push(backend.get_section());
    }
    result.sort_unstable();
    result
}

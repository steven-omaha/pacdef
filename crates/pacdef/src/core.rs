use std::collections::HashMap;
use std::env::current_dir;
use std::fs::{copy, create_dir_all, remove_file, rename, File};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, ensure, Context, Result};
use const_format::formatcp;

use crate::cmd::{run_edit_command, run_external_command};
use crate::env::{get_editor, should_print_debug_info};
use crate::grouping::group::groups_to_backend_packages;
use crate::path::{binary_in_path, get_absolutized_file_paths, get_group_dir};
use crate::prelude::*;
use crate::review::review;
use crate::search::search_packages;
use crate::ui::get_user_confirmation;

impl MainArguments {
    /// Run the action that was provided by the user as first argument.
    ///
    /// For convenience sake, all called functions take a `&self` argument, even if
    /// these are not strictly required.
    ///
    /// # Errors
    ///
    /// This function propagates errors from the underlying functions.
    pub fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        match self.subcommand {
            MainSubcommand::Group(group) => group.run(groups),
            MainSubcommand::Package(package) => package.run(groups, config),
            MainSubcommand::Version(version) => version.run(config),
        }
    }
}

impl VersionArguments {
    /// If the crate was compiled from git, return `pacdef, <version> (<hash>)`.
    /// Otherwise return `pacdef, <version>`.
    fn run(self, config: &Config) -> Result<()> {
        let backends = get_included_backends(config);
        let mut result = format!("pacdef, version: {}\n", get_version_string());
        result.push_str("supported backends:");
        for b in backends {
            result.push_str("\n  ");
            result.push_str(b);
        }

        println!("{}", result);

        Ok(())
    }
}

impl GroupArguments {
    fn run(self, groups: &Groups) -> Result<()> {
        match self.group_action {
            GroupAction::Edit(edit) => edit.run(groups),
            GroupAction::Export(export) => export.run(groups),
            GroupAction::Import(import) => import.run(),
            GroupAction::List(list) => list.run(groups),
            GroupAction::New(new) => new.run(),
            GroupAction::Remove(remove) => remove.run(groups),
            GroupAction::Show(show) => show.run(groups),
        }
    }
}

impl EditGroupAction {
    fn run(self, groups: &Groups) -> Result<()> {
        let group_files: Vec<_> = find_groups_by_name(&self.edit_groups, groups)
            .context("getting group files for args")?
            .into_iter()
            .map(|g| g.path.as_path())
            .collect();

        let mut cmd = Command::new(get_editor().context("getting suitable editor")?);
        cmd.current_dir(
            group_files[0]
                .parent()
                .context("getting parent dir of first file argument")?,
        );
        for group_file in group_files {
            cmd.arg(group_file.to_string_lossy().to_string());
        }
        run_external_command(cmd)?;

        Ok(())
    }
}

impl ExportGroupAction {
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
    fn run(self, groups: &Groups) -> Result<()> {
        let groups = find_groups_by_name(&self.export_groups, groups)?;
        let output_dir = match self.output_dir {
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
                !self.force && !exported_path.exists(),
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

impl ImportGroupAction {
    fn run(self) -> Result<()> {
        let files = get_absolutized_file_paths(&self.import_groups)?;
        let groups_dir = get_group_dir()?;

        for target in files {
            let target_name = target
                .file_name()
                .context("path should not end in '..'")?
                .to_str()
                .context("filename is not valid UTF-8")?;

            if !target.exists() {
                log::warn!("file {target_name} does not exist, skipping");
                continue;
            }

            let mut link = groups_dir.clone();
            link.push(target_name);

            if link.exists() {
                log::warn!("group {target_name} already exists, skipping");
            } else {
                symlink(target, link)?;
            }
        }

        Ok(())
    }
}

impl ListGroupAction {
    /// Print the alphabetically sorted names of all groups to stdout.
    ///
    /// This methods cannot return an error. It returns a `Result` to be consistent
    /// with other methods.
    fn run(self, groups: &Groups) -> Result<()> {
        let mut vec: Vec<_> = groups.iter().collect();
        vec.sort_unstable();
        for g in vec {
            println!("{}", g.name);
        }

        Ok(())
    }
}

impl NewGroupAction {
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
    fn run(&self) -> Result<()> {
        let group_path = get_group_dir()?;

        // prevent group names that resolve to directories
        for new_group in &self.new_groups {
            ensure!(
                new_group != "." && new_group != "..",
                Error::InvalidGroupName(new_group.clone())
            );
        }

        let paths: Vec<_> = self
            .new_groups
            .iter()
            .map(|name| {
                let mut base = group_path.clone();
                base.push(name);
                base
            })
            .collect();

        for file in &paths {
            ensure!(!file.exists(), Error::GroupAlreadyExists(file.clone()));
        }

        for file in &paths {
            File::create(file)?;
        }

        if self.edit {
            run_edit_command(&paths).context("running editor")?;
        }

        Ok(())
    }
}

impl RemoveGroupAction {
    fn run(self, groups: &Groups) -> Result<()> {
        let found = find_groups_by_name(&self.remove_groups, groups)?;

        for group in found {
            remove_file(&group.path)?;
        }

        Ok(())
    }
}

impl ShowGroupAction {
    fn run(self, groups: &Groups) -> Result<()> {
        let mut errors = vec![];
        let mut found_groups = vec![];

        // make sure all args exist before doing anything
        for show_group in &self.show_groups {
            let possible_group = groups.iter().find(|group| group.name == *show_group);

            let Some(group) = possible_group else {
                errors.push(show_group.to_string());
                continue;
            };

            found_groups.push(group);
        }

        // return an error if any arg was not found
        ensure!(errors.is_empty(), Error::MultipleGroupsNotFound(errors));

        let show_more_than_one_group = self.show_groups.len() > 1;

        let mut iter = found_groups.into_iter().peekable();

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
}

impl PackageArguments {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        match self.package_action {
            PackageAction::Clean(clean) => clean.run(groups, config),
            PackageAction::Review(review) => review.run(groups, config),
            PackageAction::Search(search) => search.run(groups),
            PackageAction::Sync(sync) => sync.run(groups, config),
            PackageAction::Unmanaged(unmanaged) => unmanaged.run(groups, config),
        }
    }
}

impl CleanPackageAction {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        let to_remove = get_unmanaged_packages(groups, config)?;

        if to_remove.nothing_to_do_for_all_backends() {
            println!("nothing to do");
            return Ok(());
        }

        println!("Would remove the following packages:\n");
        to_remove.show().context("printing things to do")?;

        println!();
        if self.no_confirm {
            println!("proceeding without confirmation");
        } else if !get_user_confirmation()? {
            return Ok(());
        }

        to_remove.remove_unmanaged_packages(self.no_confirm)
    }
}

impl ReviewPackageAction {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        review(get_unmanaged_packages(groups, config)?, groups)
    }
}

impl SearchPackageAction {
    fn run(self, groups: &Groups) -> Result<()> {
        search_packages(&self.regex, groups)
    }
}

impl SyncPackageAction {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        let to_install = get_missing_packages(groups, config)?;

        if to_install.nothing_to_do_for_all_backends() {
            println!("nothing to do");
            return Ok(());
        }

        println!("Would install the following packages:\n");
        to_install.show().context("printing things to do")?;

        println!();
        if self.no_confirm {
            println!("proceeding without confirmation");
        } else if !get_user_confirmation()? {
            return Ok(());
        }

        to_install.install_missing_packages(self.no_confirm)
    }
}

impl UnmanagedPackageAction {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        let unmanaged_per_backend = &get_unmanaged_packages(groups, config)?;

        if unmanaged_per_backend.nothing_to_do_for_all_backends() {
            return Ok(());
        }

        unmanaged_per_backend
            .show()
            .context("printing things to do")
    }
}

fn get_missing_packages(groups: &Groups, config: &Config) -> Result<ToDoPerBackend> {
    let backend_packages = groups_to_backend_packages(groups, config)?;

    let mut to_install = ToDoPerBackend::new();

    for (any_backend, packages) in &backend_packages {
        let backend_info = any_backend.backend_info();

        if config
            .disabled_backends
            .contains(&backend_info.section.to_string())
        {
            continue;
        }

        if !binary_in_path(&backend_info.binary)? {
            continue;
        }

        let managed_backend = ManagedBackend {
            packages: packages.clone(),
            any_backend: any_backend.clone(),
        };

        match managed_backend.get_missing_packages_sorted() {
            Ok(diff) => to_install.push((any_backend.clone(), diff)),
            Err(error) => show_backend_query_error(&error, any_backend),
        };
    }

    Ok(to_install)
}

/// Get a list of unmanaged packages per backend.
///
/// This method loops through all enabled `Backend`s whose binary is in `PATH`.
///
/// # Errors
///
/// This function will propagate errors from the individual backends.
fn get_unmanaged_packages(groups: &Groups, config: &Config) -> Result<ToDoPerBackend> {
    let backend_packages = groups_to_backend_packages(groups, config)?;

    let mut todo_unmanaged = ToDoPerBackend::new();

    for (any_backend, packages) in &backend_packages {
        let backend_info = any_backend.backend_info();
        if config
            .disabled_backends
            .contains(&backend_info.section.to_string())
        {
            continue;
        }

        if !binary_in_path(&backend_info.binary)? {
            continue;
        }

        let managed_backend = ManagedBackend {
            packages: packages.clone(),
            any_backend: any_backend.clone(),
        };

        match managed_backend.get_unmanaged_packages_sorted() {
            Ok(unmanaged) => todo_unmanaged.push((any_backend.clone(), unmanaged)),
            Err(error) => show_backend_query_error(&error, any_backend),
        };
    }

    Ok(todo_unmanaged)
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
            // that would be the error if permissions are okay.
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
fn find_groups_by_name<'a>(names: &[String], groups: &'a Groups) -> Result<Vec<&'a Group>> {
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

/// Show the error chain for an error that has occurred when a backend was queried
/// if the `RUST_BACKTRACE` env variable is set to `1` or `full`.
fn show_backend_query_error(error: &anyhow::Error, backend: &AnyBackend) {
    if should_print_debug_info() {
        log::warn!(
            "skipping backend '{backend}': {}",
            error.chain().map(|x| x.to_string()).collect::<String>()
        );
    } else {
        log::warn!("skipping backend '{backend}': {error}");
    }
}

/// If the crate was compiled from git, return `<version> (<hash>)`. Otherwise
/// return `<version>`.
pub const fn get_version_string() -> &'static str {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const HASH: &str = env!("GIT_HASH");

    formatcp!("{VERSION} ({HASH})")
}

/// Get a vector with the names of all backends, sorted alphabetically.
fn get_included_backends(config: &Config) -> Vec<&'static str> {
    let mut result = vec![];
    for backend in AnyBackend::all(config) {
        result.push(backend.backend_info().section);
    }
    result.sort_unstable();
    result
}

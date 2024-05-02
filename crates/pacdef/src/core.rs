use std::ffi::OsString;
use std::fs::{copy, create_dir_all, remove_file, rename};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use const_format::formatcp;

use crate::cmd::run_args;
use crate::env::{get_editor, should_print_debug_info};
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
            MainSubcommand::Clean(clean) => clean.run(groups, config),
            MainSubcommand::Review(review) => review.run(groups, config),
            MainSubcommand::Search(search) => search.run(groups),
            MainSubcommand::Sync(sync) => sync.run(groups, config),
            MainSubcommand::Unmanaged(unmanaged) => unmanaged.run(groups, config),
            MainSubcommand::Version(version) => version.run(config),
        }
    }
}

impl VersionArguments {
    /// If the crate was compiled from git, return `pacdef, <version> (<hash>)`.
    /// Otherwise return `pacdef, <version>`.
    fn run(self, _: &Config) -> Result<()> {
        println!("pacdef, version: {}\n", get_version_string());

        Ok(())
    }
}

impl CleanPackageAction {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        let unmanaged = PackagesIds::unmanaged(groups, config)?;

        if unmanaged.is_empty() {
            println!("nothing to do");
            return Ok(());
        }

        println!("Would remove the following packages:\n\n{unmanaged}\n");

        if self.no_confirm {
            println!("proceeding without confirmation");
        } else if !get_user_confirmation()? {
            return Ok(());
        }

        let packages_to_remove = PackagesRemove::from_packages_ids_defaults(&unmanaged);

        packages_to_remove.remove(self.no_confirm, config)
    }
}

impl ReviewPackageAction {
    fn run(self, _: &Groups, _: &Config) -> Result<()> {
        review()
    }
}

impl SearchPackageAction {
    fn run(self, groups: &Groups) -> Result<()> {
        search_packages(&self.regex, groups)
    }
}

impl SyncPackageAction {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        let missing = PackagesIds::missing(groups, config)?;

        if missing.is_empty() {
            println!("nothing to do");
            return Ok(());
        }

        println!("Would install the following packages:\n\n{missing}\n");

        if self.no_confirm {
            println!("proceeding without confirmation");
        } else if !get_user_confirmation()? {
            return Ok(());
        }

        let packages_to_install = PackagesInstall::from_packages_ids_defaults(&missing);

        packages_to_install.install(self.no_confirm, config)
    }
}

impl UnmanagedPackageAction {
    fn run(self, groups: &Groups, config: &Config) -> Result<()> {
        let unmanaged = PackagesIds::unmanaged(groups, config)?;

        if unmanaged.is_empty() {
            println!("no unmanaged packages");
        } else {
            println!("unmanaged packages:\n\n{unmanaged}");
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

    if HASH.is_empty() {
        VERSION
    } else {
        formatcp!("{VERSION} ({HASH})")
    }
}

fn edit_files(files: Vec<PathBuf>) -> Result<()> {
    let editor = get_editor()?;
    for file in files {
        run_args(
            [
                OsString::from(editor.as_str()),
                file.as_path().as_os_str().to_owned(),
            ]
            .into_iter(),
        )?;
    }
    Ok(())
}

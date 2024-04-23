//! Main program for `pacdef`.

#![warn(
    clippy::as_conversions,
    clippy::option_if_let_else,
    clippy::redundant_pub_crate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnecessary_wraps,
    clippy::unused_self,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::use_self,
    clippy::wildcard_dependencies,
    missing_docs
)]

use std::path::Path;
use std::process::{ExitCode, Termination};

use anyhow::{bail, Context, Result};

use clap::Parser;
use pacdef::cli::MainArguments;
use pacdef::path::{get_config_path, get_config_path_old_version, get_group_dir};
use pacdef::{Config, Error as PacdefError, Group};

const MAJOR_UPDATE_MESSAGE: &str = "VERSION UPGRADE
You seem to have used version 1.x of pacdef before.
In version 2.0 the config file needed to be changed from yaml to toml.
Check out https://github.com/steven-omaha/pacdef/blob/main/README.md#configuration for new syntax information.
This message will not appear again.
------";

fn main() -> ExitCode {
    handle_final_result(main_inner())
}

/// Skip printing the error chain when searching packages yields no results,
/// otherwise report error chain.
#[allow(clippy::option_if_let_else)]
fn handle_final_result(result: Result<()>) -> ExitCode {
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(ref e) => {
            if let Some(root_error) = e.root_cause().downcast_ref::<PacdefError>() {
                eprintln!("{root_error}");
                ExitCode::FAILURE
            } else {
                result.report()
            }
        }
    }
}

fn main_inner() -> Result<()> {
    let main_arguments = MainArguments::parse();

    let config_file = get_config_path().context("getting config file")?;

    let config = match Config::load(&config_file).context("loading config file") {
        Ok(config) => config,
        Err(e) => {
            if let Some(crate_error) = e.downcast_ref::<PacdefError>() {
                match crate_error {
                    PacdefError::ConfigFileNotFound => load_default_config(&config_file)?,
                    _ => bail!("unexpected error: {crate_error}"),
                }
            } else {
                bail!("unexpected error: {e:?}");
            }
        }
    };

    let group_dir = get_group_dir().context("resolving group dir")?;
    let groups = Group::load(&group_dir, config.warn_not_symlinks)
        .with_context(|| format!("loading groups under {}", group_dir.to_string_lossy()))?;

    for group in groups.iter() {
        if group.warn_symlink {
            eprintln!(
                "WARNING: group file {} is not a symlink",
                group.path.to_string_lossy()
            );
        }
    }

    main_arguments.run(&groups, &config)
}

fn load_default_config(config_file: &Path) -> Result<Config> {
    if get_config_path_old_version()?.exists() {
        println!("{MAJOR_UPDATE_MESSAGE}");
    }

    if !config_file.exists() {
        create_empty_config_file(config_file)?;
    }

    Ok(Config::default())
}

fn create_empty_config_file(config_file: &Path) -> Result<()> {
    let config_dir = &config_file.parent().context("getting parent dir")?;
    std::fs::create_dir_all(config_dir).context("creating parent dir")?;
    std::fs::File::create(config_file).context("creating empty config file")?;
    Ok(())
}

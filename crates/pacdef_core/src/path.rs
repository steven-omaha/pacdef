/*!
All functions related to `pacdef`'s internal paths.
*/

use std::env;
use std::path::PathBuf;

use anyhow::{Context, Result};

const CONFIG_FILE_NAME: &str = "pacdef.yaml";
const CONFIG_FILE_NAME_OLD: &str = "pacdef.conf";

/// Get the group directory where all group files are located. This is
/// `$XDG_CONFIG_HOME/pacdef/groups`, which defaults to `$HOME/.config/pacdef/groups`.
///
/// # Errors
///
/// This function returns an error if both `$XDG_CONFIG_HOME` and `$HOME` are undefined.
pub fn get_group_dir() -> Result<PathBuf> {
    let mut result = get_pacdef_base_dir().context("getting pacdef base dir")?;
    result.push("groups");
    Ok(result)
}

pub(crate) fn get_pacdef_base_dir() -> Result<PathBuf> {
    let mut dir = get_xdg_config_home().context("getting XDG_CONFIG_HOME")?;
    dir.push("pacdef");
    Ok(dir)
}

fn get_xdg_config_home() -> Result<PathBuf> {
    if let Ok(config) = env::var("XDG_CONFIG_HOME") {
        Ok(config.into())
    } else {
        let mut config = get_home_dir().context("falling back to $HOME/.config")?;
        config.push(".config");
        Ok(config)
    }
}

pub(crate) fn get_home_dir() -> Result<PathBuf> {
    Ok(env::var("HOME").context("getting $HOME variable")?.into())
}

/// Get the path to the pacdef config file. This is `$XDG_CONFIG_HOME/pacdef/pacdef.yaml`.
///
/// # Errors
///
/// This function returns an error if both `$XDG_CONFIG_HOME` and `$HOME` are undefined.
pub fn get_config_path() -> Result<PathBuf> {
    let mut file = get_pacdef_base_dir().context("getting pacdef base dir for config file")?;
    file.push(CONFIG_FILE_NAME);
    Ok(file)
}

/// Get the path to the pacdef config file from version 0.x. This is
/// `$XDG_CONFIG_HOME/pacdef/pacdef.yaml`.
///
/// # Errors
///
/// This function returns an error if both `$XDG_CONFIG_HOME` and `$HOME` are
/// undefined.
pub fn get_config_path_old_version() -> Result<PathBuf> {
    let mut file = get_pacdef_base_dir().context("getting pacdef base dir for config file")?;
    file.push(CONFIG_FILE_NAME_OLD);
    Ok(file)
}

pub(crate) fn binary_in_path(name: &str) -> Result<bool> {
    let paths = env::var_os("PATH").context("getting $PATH")?;
    for dir in env::split_paths(&paths) {
        let full_path = dir.join(name);
        if full_path.is_file() {
            return Ok(true);
        }
    }
    Ok(false)
}

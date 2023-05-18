/*!
All functions related to `pacdef`'s internal paths.
*/

use std::path::PathBuf;
use std::{env, path::Path};

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

/// Get the base directory for `pacdef`'s config files.
///
/// # Errors
///
/// This function will return an error if `$XDG_CONFIG_HOME` cannot be determined.
pub(crate) fn get_pacdef_base_dir() -> Result<PathBuf> {
    let mut dir = get_xdg_config_home().context("getting XDG_CONFIG_HOME")?;
    dir.push("pacdef");
    Ok(dir)
}

/// Get the path to the XDG config directory.
///
/// # Errors
///
/// This function will return an error if neither the `$XDG_CONFIG_HOME` nor
/// the `$HOME` environment variables are set.
fn get_xdg_config_home() -> Result<PathBuf> {
    if let Ok(config) = env::var("XDG_CONFIG_HOME") {
        Ok(config.into())
    } else {
        let mut config = get_home_dir().context("falling back to $HOME/.config")?;
        config.push(".config");
        Ok(config)
    }
}

/// Get the home directory of the current user from the `$HOME` environment
/// variable.
///
/// # Errors
///
/// This function will return an error if the `$HOME` variable is not set.
pub(crate) fn get_home_dir() -> Result<PathBuf> {
    Ok(env::var("HOME").context("getting $HOME variable")?.into())
}

/// Get the path to the pacdef config file. This is `$XDG_CONFIG_HOME/pacdef/pacdef.yaml`.
///
/// # Errors
///
/// This function returns an error if both `$XDG_CONFIG_HOME` and `$HOME` are
/// undefined.
pub fn get_config_path() -> Result<PathBuf> {
    let mut file = get_pacdef_base_dir().context("getting pacdef base dir for config file")?;
    file.push(CONFIG_FILE_NAME);
    Ok(file)
}

/// Get the path to the pacdef config file from version 0.x. This is
/// `$XDG_CONFIG_HOME/pacdef/pacdef.conf`.
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

/// Determine if a program `name` exists in the folders in the `$PATH` variable.
///
/// # Errors
///
/// This function returns an error if `$PATH` is not set.
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

/// Determine the relative path of `full_path` in relation to `base_path`.
///
/// # Panics
///
/// Panics if at least one element in `base_path` does not match the corresponding
/// element in `full_path`.
pub(crate) fn get_relative_path<P>(full_path: P, base_path: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let mut file_iter = full_path.as_ref().iter();
    base_path
        .as_ref()
        .iter()
        .zip(&mut file_iter)
        .for_each(|(a, b)| assert_eq!(a, b));
    let relative_path: PathBuf = file_iter.collect();
    relative_path
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::get_relative_path;

    #[test]
    fn relative_path() {
        let full = PathBuf::from("/a/b/c/d/e");
        let base = PathBuf::from("/a/b/c");
        let relative = get_relative_path(full, base);
        assert_eq!(relative, PathBuf::from("d/e"));
    }

    #[test]
    #[should_panic]
    fn relative_path_panic() {
        let full = PathBuf::from("/a/b/z/d/e");
        let base = PathBuf::from("/a/b/c");
        get_relative_path(full, base);
    }
}

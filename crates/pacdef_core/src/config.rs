use std::fs::{read_to_string, File};
use std::io::{ErrorKind, Write};
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_derive::{Deserialize, Serialize};

/// Config for the program, as listed in `$XDG_CONFIG_HOME/pacdef/pacdef.yaml`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The AUR helper to use for Arch Linux.
    pub aur_helper: String,
    /// Additional arguments to pass to `aur_helper` when removing a package.
    pub aur_rm_args: Option<Vec<String>>,
    /// Warn the user when a group is not a symlink.
    pub warn_not_symlinks: bool,
}

impl Config {
    /// Load the config from the associated file. If the file does not exist, create a default config.
    ///
    /// # Errors
    ///
    /// This function will return an error if the config file exists but cannot be read, its contents are not UTF-8, or the file is  malformed.
    pub fn load(config_file: &Path) -> Result<Self> {
        let from_file = read_to_string(config_file);

        let content = match from_file {
            Ok(content) => content,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    println!(
                        "creating default config under {}",
                        config_file.to_string_lossy()
                    );
                    return Self::use_default_and_save_to(config_file);
                }
                bail!("unexpected error occured: {e:?}");
            }
        };

        serde_yaml::from_str(&content).context("parsing yaml config")
    }

    fn use_default_and_save_to(file: &Path) -> Result<Self> {
        let result = Self::default();

        let content = serde_yaml::to_string(&result).context("converting Config to yaml")?;
        let mut output = File::create(file).context("creating default config file")?;
        write!(output, "{content}").context("writing default config")?;

        Ok(result)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aur_helper: "paru".into(),
            aur_rm_args: None,
            warn_not_symlinks: true,
        }
    }
}

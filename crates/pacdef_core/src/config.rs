use std::fs::{create_dir_all, read_to_string, File};
use std::io::{ErrorKind, Write};
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_derive::{Deserialize, Serialize};

// Update the master README if fields change.
/// Config for the program, as listed in `$XDG_CONFIG_HOME/pacdef/pacdef.yaml`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The AUR helper to use for Arch Linux.
    #[serde(default = "aur_helper")]
    pub aur_helper: String,
    /// Additional arguments to pass to `aur_helper` when removing a package.
    #[serde(default)]
    pub aur_rm_args: Vec<String>,
    /// Install Flatpak packages system-wide
    #[serde(default = "yes")]
    pub flatpak_systemwide: bool,
    /// Warn the user when a group is not a symlink.
    #[serde(default = "yes")]
    pub warn_not_symlinks: bool,
    /// Backends the user does not want to use even though the binary exists.
    #[serde(default)]
    pub disabled_backends: Vec<String>,
    /// Choose whether to use pipx instead of pip for python package management
    #[serde(default)]
    pub pip_binary: String,
}

fn yes() -> bool {
    true
}

fn aur_helper() -> String {
    "paru".into()
}

impl Config {
    /// Load the config from the associated file.
    ///
    /// # Errors
    ///
    /// This function will return an error if the config file exists but cannot be
    /// read, its contents are not UTF-8, or the file is malformed.
    pub fn load(config_file: &Path) -> Result<Self> {
        let from_file = read_to_string(config_file);

        let content = match from_file {
            Ok(content) => content,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    bail!(crate::Error::ConfigFileNotFound)
                }
                bail!("unexpected error occured: {e:?}");
            }
        };

        serde_yaml::from_str(&content).context("parsing yaml config")
    }

    /// Save the instance of [`Config`] to disk.
    ///
    /// # Errors
    ///
    /// This function will return an error if the config file cannot be saved to disk.
    pub fn save(&self, file: &Path) -> Result<()> {
        let content = serde_yaml::to_string(&self).context("converting Config to yaml")?;

        let parent = file.parent().context("getting parent of config dir")?;
        if !parent.is_dir() {
            create_dir_all(parent)
                .with_context(|| format!("creating dir {}", parent.to_string_lossy()))?;
        }

        let mut output = File::create(file).context("creating default config file")?;
        write!(output, "{content}").context("writing default config")?;

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aur_helper: "paru".into(),
            aur_rm_args: vec![],
            flatpak_systemwide: true,
            warn_not_symlinks: true,
            disabled_backends: vec![],
            pip_binary: "pip".into(),
        }
    }
}

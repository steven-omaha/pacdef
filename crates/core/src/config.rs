use std::fs::{read_to_string, File};
use std::io::{ErrorKind, Write};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde_derive::{Deserialize, Serialize};

use crate::path::get_pacdef_base_dir;

const CONFIG_FILE_NAME: &str = "pacdef.yaml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub aur_helper: String,
    pub aur_rm_args: Option<Vec<String>>,
    pub warn_not_symlinks: bool,
}

impl Config {
    pub fn load() -> Result<Self> {
        let file = get_config_file()?;
        let from_file = read_to_string(&file);

        if let Err(e) = from_file {
            if e.kind() == ErrorKind::NotFound {
                println!("creating default config under {file:?}");
                return Self::use_default_and_save_to(file);
            } else {
                bail!("unexpected error occured: {e:?}");
            };
        }

        let content = from_file.expect("we already handled that error");

        serde_yaml::from_str(&content).context("parsing yaml config")
    }

    fn use_default_and_save_to(file: PathBuf) -> Result<Self> {
        let result = Self::default();

        let content = serde_yaml::to_string(&result).context("converting Config to yaml")?;
        let mut output = File::create(file).context("creating default config file")?;
        write!(output, "{content}").context("writing default config")?;

        Ok(result)
    }
}

fn get_config_file() -> Result<PathBuf> {
    let mut file = get_pacdef_base_dir().context("getting pacdef base dir for config file")?;
    file.push(CONFIG_FILE_NAME);
    Ok(file)
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

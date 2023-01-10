use std::{env, path::PathBuf};

use anyhow::{Context, Result};

pub(crate) fn get_pacdef_group_dir() -> Result<PathBuf> {
    let mut result = get_pacdef_base_dir().context("getting pacdef base dir")?;
    result.push("groups");
    Ok(result)
}

fn get_pacdef_base_dir() -> Result<PathBuf> {
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

use std::path::Path;
use std::process::{Command, ExitStatus};

use anyhow::{anyhow, Context, Result};

use crate::env::get_editor;

pub fn run_edit_command<P>(files: &[P]) -> Result<ExitStatus>
where
    P: for<'a> AsRef<&'a Path>,
{
    let mut cmd = Command::new(get_editor().context("getting suitable editor")?);
    cmd.current_dir(
        files[0]
            .as_ref()
            .parent()
            .context("getting parent dir of first file argument")?,
    );
    for f in files {
        cmd.arg(f.as_ref().to_string_lossy().to_string());
    }
    cmd.status().map_err(|e| anyhow!(e))
}

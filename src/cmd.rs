use std::path::PathBuf;
use std::process::{Command, ExitStatus};

use anyhow::{anyhow, Result};

use crate::env::get_editor;

pub fn run_edit_command(files: &[PathBuf]) -> Result<ExitStatus> {
    let mut cmd = Command::new(get_editor()?);
    for f in files {
        cmd.arg(f.to_string_lossy().to_string());
    }
    cmd.status().map_err(|e| anyhow!(e))
}

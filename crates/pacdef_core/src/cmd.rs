use std::path::Path;
use std::process::{Command, ExitStatus};

use anyhow::{anyhow, Context, Result};

use crate::env::get_editor;

/// Run the editor and pass the provided files as arguments. The workdir is set
/// to the parent of the first file.
pub fn run_edit_command<P>(files: &[P]) -> Result<ExitStatus>
where
    P: AsRef<Path>,
{
    fn inner(files: &[&Path]) -> Result<ExitStatus> {
        let mut cmd = Command::new(get_editor().context("getting suitable editor")?);
        cmd.current_dir(
            files[0]
                .parent()
                .context("getting parent dir of first file argument")?,
        );
        for f in files {
            cmd.arg(f.to_string_lossy().to_string());
        }
        cmd.status().map_err(|e| anyhow!(e))
    }

    let files: Vec<_> = files.iter().map(|p| p.as_ref()).collect();
    inner(&files)
}

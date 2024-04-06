use std::path::Path;
use std::process::{Command, ExitStatus};

use anyhow::{anyhow, ensure, Context, Result};

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
        // TODO this could also use the run_external_command function
        cmd.status().map_err(|e| anyhow!(e))
    }

    let files: Vec<_> = files.iter().map(|p| p.as_ref()).collect();
    inner(&files)
}

/// Run an external command. Use the anyhow framework to bubble up errors if they occur.
///
/// # Errors
///
/// This function will return an error if the command cannot be run or if it returns a non-zero
/// exit status. In case of an error the full command will be part of the error message.
pub fn run_external_command(mut cmd: Command) -> Result<()> {
    let exit_status = cmd.status().with_context(|| "running command [{cmd:?}]")?;
    let success = exit_status.success();
    ensure!(
        success,
        "command [{cmd:?}] returned non-zero exit status {success}"
    );
    Ok(())
}

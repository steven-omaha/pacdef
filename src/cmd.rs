use std::process::Command;

use anyhow::Result;

pub fn command_found(command: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for p in path.split(':') {
            let p_str = format!("{}/{}", p, command);
            if std::fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

pub fn run_args_for_stdout<S>(mut args: impl Iterator<Item = S>) -> Result<String>
where
    S: std::convert::AsRef<std::ffi::OsStr>,
{
    let we_are_root = {
        let uid = unsafe { libc::geteuid() };
        uid == 0
    };

    let mut cmd = if we_are_root {
        Command::new("sudo")
    } else {
        Command::new(args.next().expect("cannot run an empty set of args"))
    };

    cmd.args(args);

    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(anyhow::anyhow!("command failed"))
    }
}

pub fn run_args<S>(args: impl Iterator<Item = S>) -> Result<()>
where
    S: std::convert::AsRef<std::ffi::OsStr>,
{
    run_args_for_stdout(args).map(|_| ())
}
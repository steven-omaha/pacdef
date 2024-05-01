use std::process::Command;

use anyhow::Result;

pub fn run_args_for_stdout<'a>(mut args: impl Iterator<Item = &'a str>) -> Result<String> {
    let we_are_root = {
        let uid = unsafe { libc::geteuid() };
        uid == 0
    };

    let mut cmd = if we_are_root {
        Command::new("sudo")
    } else {
        Command::new(args.next().unwrap())
    };

    cmd.args(args);

    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(anyhow::anyhow!("command failed"))
    }
}

pub fn run_args<'a>(args: impl Iterator<Item = &'a str>) -> Result<()> {
    run_args_for_stdout(args).map(|_| ())
}

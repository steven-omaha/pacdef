use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

use crate::Package;

pub fn run_edit_command(files: &[PathBuf]) {
    let mut cmd = Command::new("nvim");
    for f in files {
        cmd.arg(f.to_string_lossy().to_string());
    }
    cmd.exec();
}

pub fn run_install_command(diff: Vec<Package>) {
    let mut cmd = Command::new("paru");
    cmd.arg("-S");
    for p in diff {
        cmd.arg(format!("{p}"));
    }
    cmd.exec();
}

use std::process::Command;

pub fn we_are_root() -> bool {
    let uid = unsafe { libc::geteuid() };
    uid == 0
}

pub fn build_base_command_with_privileges(binary: &str) -> Command {
    let cmd = if we_are_root() {
        Command::new(binary)
    } else {
        let mut cmd = Command::new("sudo");
        cmd.arg(binary);
        cmd
    };
    cmd
}

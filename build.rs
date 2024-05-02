use std::process::Command;

fn main() {
    let git_hash = match Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        Ok(output) => String::from_utf8(output.stdout).expect("git output is utf-8"),
        _ => String::new(),
    };

    println!("cargo::rustc-env=GIT_HASH={git_hash}");
}

use std::io::BufRead;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::process::exit;
use std::{collections::HashSet, process::Command};

use alpm::Alpm;
use args::get_arg_parser;
use group::Group;
use package::Package;

mod args;
pub(crate) mod group;
pub(crate) mod package;

fn main() {
    let args = get_arg_parser();
    let matches = args.get_matches();
    match matches.subcommand() {
        Some(("sync", _)) => install_pacdef_packages(),
        s => panic!("{:#?}", s),
    }
}

fn install_pacdef_packages() {
    let groups = Group::load_from_dir();
    let packages_hs = groups
        .into_iter()
        .flat_map(|g| g.packages)
        .collect::<HashSet<_>>();

    let local_packages = convert_to_pacdef_packages(get_alpm_packages());
    let mut diff: Vec<_> = packages_hs.difference(&local_packages).collect();
    diff.sort_unstable();
    if diff.is_empty() {
        exit(0);
    }
    println!("Would install the following packages:");
    for p in &diff {
        println!("  {p}");
    }
    println!();
    get_user_confirmation();

    install_packages(diff);
}

fn get_user_confirmation() {
    print!("Continue? [Y/n] ");
    std::io::stdout().flush().unwrap();
    let reply = std::io::stdin().lock().lines().next().unwrap().unwrap();
    if !(reply.is_empty() || reply.to_lowercase().contains('y')) {
        exit(0)
    }
}

fn get_alpm_packages() -> HashSet<String> {
    let db = Alpm::new("/", "/var/lib/pacman").unwrap();
    db.localdb()
        .pkgs()
        .iter()
        .map(|p| p.name().to_string())
        .collect()
}

fn convert_to_pacdef_packages(packages: HashSet<String>) -> HashSet<Package> {
    packages.into_iter().map(Package::from).collect()
}

fn install_packages(diff: Vec<&Package>) {
    let mut cmd = Command::new("paru");
    cmd.arg("-S");
    for p in diff {
        cmd.arg(format!("{p}"));
    }
    dbg!(&cmd);
    cmd.exec();
}

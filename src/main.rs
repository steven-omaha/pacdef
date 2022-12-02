use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{collections::HashSet, process::Command};

use anyhow::Result;
use clap::ArgMatches;

use pacdef::args;
use pacdef::db::{get_all_installed_packages, get_explicitly_installed_packages};
use pacdef::group::GROUPS_DIR;
use pacdef::Group;
use pacdef::Package;

fn main() -> Result<()> {
    let args = args::get_matched_arguments();
    let groups = Group::load_from_dir();
    let pacdef = Pacdef::new(args, groups);
    pacdef.run_action_from_arg();
    Ok(())
}

fn run_edit_command(files: &[&Path]) {
    let mut cmd = Command::new("nvim");
    for f in files {
        cmd.arg(f.to_string_lossy().to_string());
    }
    dbg!(&cmd);
    cmd.exec();
}

fn run_install_command(diff: Vec<Package>) {
    let mut cmd = Command::new("paru");
    cmd.arg("-S");
    for p in diff {
        cmd.arg(format!("{p}"));
    }
    dbg!(&cmd);
    cmd.exec();
}

struct Pacdef {
    args: ArgMatches,
    groups: Option<HashSet<Group>>,
    // action: Box<dyn Fn(Self)>,
}

impl Pacdef {
    fn new(args: ArgMatches, groups: HashSet<Group>) -> Self {
        Self {
            args,
            groups: Some(groups),
            // action: Box::new(Self::install_packages),
        }
    }

    fn take_packages_as_set(&mut self) -> HashSet<Package> {
        self.groups
            .take()
            .unwrap()
            .into_iter()
            .flat_map(|g| g.packages)
            .collect()
    }

    fn get_packages_to_install(&mut self) -> Vec<Package> {
        let managed = self.take_packages_as_set();
        let local_packages = get_all_installed_packages();
        let mut diff: Vec<_> = managed
            .into_iter()
            .filter(|p| !local_packages.contains(p))
            .collect();
        diff.sort_unstable();
        diff
    }

    fn install_packages(mut self) {
        let diff = self.get_packages_to_install();
        if diff.is_empty() {
            println!("nothing to do");
            exit(0);
        }
        println!("Would install the following packages:");
        for p in &diff {
            println!("  {p}");
        }
        println!();
        pacdef::ui::get_user_confirmation();

        run_install_command(diff);
    }

    fn run_action_from_arg(self) {
        match self.args.subcommand() {
            // Some((args::EDIT, groups)) => println!("{groups:#?}"),
            Some((args::EDIT, groups)) => self.edit_group_files(groups),
            Some((args::GROUPS, _)) => self.show_groups(),
            Some((args::SYNC, _)) => self.install_packages(),
            Some((args::UNMANAGED, _)) => self.show_unmanaged_packages(),
            Some((args::VERSION, _)) => self.show_version(),
            _ => todo!(),
        }
    }

    fn edit_group_files(&self, groups: &ArgMatches) {
        let files: Vec<_> = groups
            .get_many::<String>("group")
            .unwrap()
            .map(|file| {
                let mut buf = PathBuf::from(GROUPS_DIR);
                buf.push(file);
                buf
            })
            .collect();
        for f in files {
            println!("{f:#?}");
        }
    }

    fn show_version(self) {
        println!("pacdef, version: {}", env!("CARGO_PKG_VERSION"))
    }

    fn show_unmanaged_packages(mut self) {
        for p in &self.get_unmanaged_packages() {
            println!("{p}");
        }
    }

    /// Returns a `Vec` of alphabetically sorted unmanaged packages.
    fn get_unmanaged_packages(&mut self) -> Vec<Package> {
        let managed = self.take_packages_as_set();
        let explicitly_installed = get_explicitly_installed_packages();
        let mut result: Vec<_> = explicitly_installed
            .into_iter()
            .filter(|p| !managed.contains(p))
            .collect();
        result.sort_unstable();
        result
    }

    fn show_groups(mut self) {
        let groups = self.groups.take().unwrap();
        let mut vec: Vec<_> = groups.iter().collect();
        vec.sort_unstable();
        for g in vec {
            println!("{}", g.name);
        }
    }
}

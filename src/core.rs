use std::collections::HashSet;

use anyhow::{bail, Context, Result};
use clap::ArgMatches;

use crate::action;
use crate::backend::{Backend, Backends};
use crate::cmd::run_edit_command;
use crate::ui::get_user_confirmation;
use crate::Group;
use crate::Package;

pub struct Pacdef {
    args: ArgMatches,
    groups: HashSet<Group>,
}

impl Pacdef {
    pub fn new(args: ArgMatches, groups: HashSet<Group>) -> Self {
        Self { args, groups }
    }

    fn install_packages(&self) {
        let mut to_install = ToDoPerBackend::new();

        for mut b in Backends::iter() {
            print!("{}: ", b.get_binary());

            b.load(&self.groups);

            let diff = b.get_missing_packages_sorted();
            if diff.is_empty() {
                println!("nothing to do");
                continue;
            }

            println!("would install the following packages");
            for p in &diff {
                println!("  {p}");
            }
            to_install.push((b, diff));
            println!();
        }

        if to_install.nothing_to_do_for_all_backends() {
            return;
        }

        if !get_user_confirmation() {
            return;
        };

        to_install.install_missing_packages()
    }

    #[allow(clippy::unit_arg)]
    pub fn run_action_from_arg(self) -> Result<()> {
        match self.args.subcommand() {
            Some((action::CLEAN, _)) => Ok(self.clean_packages()),
            Some((action::EDIT, groups)) => {
                self.edit_group_files(groups).context("editing group files")
            }
            Some((action::GROUPS, _)) => Ok(self.show_groups()),
            Some((action::SYNC, _)) => Ok(self.install_packages()),
            Some((action::UNMANAGED, _)) => Ok(self.show_unmanaged_packages()),
            Some((action::VERSION, _)) => Ok(self.show_version()),
            Some((_, _)) => todo!(),
            None => unreachable!(),
        }
    }

    fn edit_group_files(&self, groups: &ArgMatches) -> Result<()> {
        let files: Vec<_> = groups
            .get_many::<String>("group")
            .context("getting group from args")?
            .map(|file| {
                let mut buf = crate::path::get_pacdef_group_dir().unwrap();
                buf.push(file);
                buf
            })
            .collect();

        for file in &files {
            if !file.exists() {
                bail!("group file {} not found", file.to_string_lossy());
            }
        }

        if run_edit_command(&files)
            .context("running editor")?
            .success()
        {
            Ok(())
        } else {
            bail!("editor exited with error")
        }
    }

    fn show_version(self) {
        println!("pacdef, version: {}", env!("CARGO_PKG_VERSION"))
    }

    fn show_unmanaged_packages(self) {
        let unmanaged_per_backend = &self.get_unmanaged_packages();

        for (backend, packages) in unmanaged_per_backend.iter() {
            if packages.is_empty() {
                continue;
            }
            println!("{}", backend.get_section());
            for package in packages {
                println!("  {package}");
            }
        }
    }

    fn get_unmanaged_packages(self) -> ToDoPerBackend {
        let mut result = ToDoPerBackend::new();

        for mut backend in Backends::iter() {
            backend.load(&self.groups);

            let unmanaged = backend.get_unmanaged_packages_sorted();
            result.push((backend, unmanaged));
        }
        result
    }

    fn show_groups(self) {
        let mut vec: Vec<_> = self.groups.iter().collect();
        vec.sort_unstable();
        for g in vec {
            println!("{}", g.name);
        }
    }

    fn clean_packages(self) {
        let to_remove = self.get_unmanaged_packages();
        if to_remove.is_empty() {
            println!("nothing to do");
            return;
        }

        println!("Would remove the following packages and their dependencies:");
        for (backend, packages) in to_remove.iter() {
            if packages.is_empty() {
                continue;
            }

            println!("  {}", backend.get_section());
            for package in packages {
                println!("    {}", package);
            }
        }

        if !get_user_confirmation() {
            return;
        };

        for (backend, packages) in to_remove.into_iter() {
            backend.remove_packages(packages);
        }
    }
}

struct ToDoPerBackend(Vec<(Box<dyn Backend>, Vec<Package>)>);

impl ToDoPerBackend {
    fn new() -> Self {
        Self(vec![])
    }

    fn push(&mut self, item: (Box<dyn Backend>, Vec<Package>)) {
        self.0.push(item);
    }

    fn into_iter(self) -> impl Iterator<Item = (Box<dyn Backend>, Vec<Package>)> {
        self.0.into_iter()
    }

    fn iter(&self) -> impl Iterator<Item = &(Box<dyn Backend>, Vec<Package>)> {
        self.0.iter()
    }

    fn nothing_to_do_for_all_backends(&self) -> bool {
        self.0.iter().all(|(_, diff)| diff.is_empty())
    }

    fn install_missing_packages(&self) {
        self.0
            .iter()
            .for_each(|(backend, diff)| backend.install_packages(diff));
    }

    fn is_empty(&self) -> bool {
        self.0.iter().all(|(_, packages)| packages.is_empty())
    }
}

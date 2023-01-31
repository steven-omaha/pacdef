use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::process::Command;
use std::rc::Rc;
use std::{collections::HashSet, process::ExitStatus};

use anyhow::{Context, Result};

use crate::{Group, Package};

pub(in crate::backend) type Switches = &'static [&'static str];
pub(in crate::backend) type Text = &'static str;

pub(crate) trait Backend: Debug {
    fn get_binary(&self) -> Text;
    fn get_section(&self) -> Text;

    fn get_switches_info(&self) -> Switches;
    fn get_switches_install(&self) -> Switches;
    fn get_switches_remove(&self) -> Switches;
    fn get_switches_make_dependency(&self) -> Switches;

    fn load(&mut self, groups: &HashSet<Group>);

    fn get_managed_packages(&self) -> &HashSet<Package>;

    /// Get all packages that are installed in the system.
    fn get_all_installed_packages(&self) -> Result<HashSet<Package>>;

    /// Get all packages that were installed in the system explicitly.
    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>>;

    fn assign_group(&self, to_assign: Vec<(Package, Rc<Group>)>) {
        todo!();
        let mut group_package_map = HashMap::new();

        for (p, group) in to_assign {
            if !group_package_map.contains_key(&group) {
                group_package_map.insert(group.clone(), vec![]);
            }

            let inner = group_package_map.get_mut(&group).unwrap();
            inner.push(p);
        }

        for vecs in group_package_map.values_mut() {
            vecs.sort();
        }

        for (group, packages) in group_package_map {
            let group_file_content = read_to_string(&group.path).unwrap();
            let old_length = dbg!(group_file_content.len());
            let mut must_write_section_header = false;

            let mut lines = group_file_content.lines();

            lines.find(|line| line.contains(&format!("[{}]", self.get_section())));
            lines.next().unwrap();

            // let start_of_section = if let Some(start_of_section) = group_file_content.find() {
            //     start_of_section
            // } else {
            //     must_write_section_header = true;
            //     old_length
            // };

            // dbg!(&start_of_section);

            // let mut new_file_content = group_file_content
            //     .get(..start_of_section)
            //     .unwrap()
            //     .to_owned();

            // dbg!(&new_file_content);

            // todo!();

            // if dbg!(must_write_section_header) {
            //     new_file_content.push_str(&format!("\n[{}]", self.get_section()));
            // }
            // for package in packages {
            //     new_file_content.push_str(&format!("\n{package}"));
            // }
            // new_file_content.push('\n');

            // if old_length > start_of_section {
            //     new_file_content.push_str(group_file_content.get(old_length..).unwrap());
            // }

            // let mut file = File::create(&group.path).unwrap();
            // write!(file, "{new_file_content}").unwrap();
        }
    }

    /// Install the specified packages.
    fn install_packages(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_install());
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.status()
            .with_context(|| format!("running command {cmd:?}"))
    }

    fn make_dependency(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_make_dependency());
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }

    /// Remove the specified packages.
    fn remove_packages(&self, packages: &[Package]) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_remove());
        for p in packages {
            cmd.arg(format!("{p}"));
        }
        cmd.status()
            .with_context(|| format!("running command [{cmd:?}]"))
    }

    /// extract packages from its own section as read from group files
    fn extract_packages_from_group_file_content(&self, content: &str) -> HashSet<Package> {
        content
            .lines()
            .skip_while(|line| !line.starts_with(&format!("[{}]", self.get_section())))
            .skip(1)
            .filter(|line| !line.starts_with('['))
            .fuse()
            .filter_map(Package::try_from)
            .collect()
    }

    fn get_missing_packages_sorted(&self) -> Result<Vec<Package>> {
        let installed = self
            .get_all_installed_packages()
            .context("could not get installed packages")?;
        let managed = self.get_managed_packages();
        let mut diff: Vec<_> = managed.difference(&installed).cloned().collect();
        diff.sort_unstable();
        Ok(diff)
    }

    fn add_packages(&mut self, packages: HashSet<Package>);

    /// Show information from package manager for package.
    fn show_package_info(&self, package: &Package) -> Result<ExitStatus> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(self.get_switches_info());
        cmd.arg(format!("{package}"));
        cmd.status()
            .with_context(|| format!("running command {cmd:?}"))
    }

    fn get_unmanaged_packages_sorted(&self) -> Result<Vec<Package>> {
        let installed = self
            .get_explicitly_installed_packages()
            .context("could not get explicitly installed packages")?;
        let required = self.get_managed_packages();
        let mut diff: Vec<_> = installed.difference(required).cloned().collect();
        diff.sort_unstable();
        Ok(diff)
    }
}

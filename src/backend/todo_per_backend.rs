use anyhow::{bail, ensure, Context, Result};

use super::Backend;
use crate::Package;

pub(crate) struct ToDoPerBackend(Vec<(Box<dyn Backend>, Vec<Package>)>);

impl ToDoPerBackend {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }

    pub(crate) fn push(&mut self, item: (Box<dyn Backend>, Vec<Package>)) {
        self.0.push(item);
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(Box<dyn Backend>, Vec<Package>)> {
        self.0.iter()
    }

    pub(crate) fn nothing_to_do_for_all_backends(&self) -> bool {
        self.0.iter().all(|(_, diff)| diff.is_empty())
    }

    // TODO try to combine these methods into one
    pub(crate) fn install_missing_packages(&self) -> Result<()> {
        for (backend, packages) in &self.0 {
            let exit_status = backend
                .install_packages(packages)
                .with_context(|| format!("installing packages for {}", backend.get_binary()))?;

            match exit_status.code() {
                Some(val) => ensure!(val == 0, "command returned with exit code {val}"),
                None => bail!("could not install packages for {}", backend.get_binary()),
            }
        }
        Ok(())
    }

    // TODO this one
    pub(crate) fn remove_unmanaged_packages(&self) -> Result<()> {
        for (backend, packages) in &self.0 {
            let exit_status = backend
                .remove_packages(packages)
                .with_context(|| format!("removing packages for {}", backend.get_binary()))?;

            match exit_status.code() {
                Some(val) => ensure!(val == 0, "command returned with exit code {val}"),
                None => bail!("could not remove packages for {}", backend.get_binary()),
            }
        }
        Ok(())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.iter().all(|(_, packages)| packages.is_empty())
    }

    pub(crate) fn show(&self, keyword: Option<&str>) {
        for (backend, packages) in self.iter() {
            if packages.is_empty() {
                continue;
            }

            if let Some(kw) = keyword {
                println!("Would {kw} the following packages:");
            }
            println!("[{}]", backend.get_section());
            for package in packages {
                println!("{package}");
            }
        }
    }
}

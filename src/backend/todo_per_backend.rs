use std::process::ExitStatus;

use anyhow::{bail, ensure, Context, Result};

use super::Backend;
use crate::Package;

#[derive(Debug)]
pub(crate) struct ToDoPerBackend(Vec<(Box<dyn Backend>, Vec<Package>)>);

impl ToDoPerBackend {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }

    pub(crate) fn push(&mut self, item: (Box<dyn Backend>, Vec<Package>)) {
        self.0.push(item);
    }

    pub(crate) fn into_iter(self) -> impl Iterator<Item = (Box<dyn Backend>, Vec<Package>)> {
        self.0.into_iter()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(Box<dyn Backend>, Vec<Package>)> {
        self.0.iter()
    }

    pub(crate) fn nothing_to_do_for_all_backends(&self) -> bool {
        self.0.iter().all(|(_, diff)| diff.is_empty())
    }

    pub(crate) fn install_missing_packages(&self) -> Result<()> {
        self.handle_backend_command(Backend::install_packages, "install", "installing")
    }

    pub(crate) fn remove_unmanaged_packages(&self) -> Result<()> {
        self.handle_backend_command(Backend::remove_packages, "remove", "removing")
    }

    fn handle_backend_command<'a, F>(
        &'a self,
        func: F,
        verb: &'static str,
        verb_continuous: &'static str,
    ) -> Result<()>
    where
        F: Fn(&'a dyn Backend, &'a [Package]) -> Result<ExitStatus>,
    {
        for (backend, packages) in &self.0 {
            if packages.is_empty() {
                continue;
            }
            let exit_status = func(&**backend, packages).with_context(|| {
                format!("{verb_continuous} packages for {}", backend.get_binary())
            })?;

            match exit_status.code() {
                Some(val) => ensure!(val == 0, "command returned with exit code {val}"),
                None => bail!("could not {verb} packages for {}", backend.get_binary()),
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

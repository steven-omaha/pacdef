use std::fmt::Write;
use std::process::ExitStatus;

use anyhow::{bail, ensure, Context, Result};

use super::Backend;
use crate::Package;

/// A vector of tuples containing a `dyn Backend` and a vector of unmanaged packages
/// for that backend.
///
/// This struct is used to store a list of unmanaged packages or missing packages
/// for all backends.
#[derive(Debug)]
pub struct ToDoPerBackend(Vec<(Box<dyn Backend>, Vec<Package>)>);

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

    pub(crate) fn install_missing_packages(&self, noconfirm: bool) -> Result<()> {
        self.handle_backend_command(
            Backend::install_packages,
            noconfirm,
            "install",
            "installing",
        )
        .context("installing packages")
    }

    pub(crate) fn remove_unmanaged_packages(&self, noconfirm: bool) -> Result<()> {
        self.handle_backend_command(Backend::remove_packages, noconfirm, "remove", "removing")
            .context("removing packages")
    }

    fn handle_backend_command<'a, F>(
        &'a self,
        func: F,
        noconfirm: bool,
        verb: &'_ str,
        verb_continuous: &'_ str,
    ) -> Result<()>
    where
        F: Fn(&'a dyn Backend, &'a [Package], bool) -> Result<ExitStatus>,
    {
        for (backend, packages) in &self.0 {
            if packages.is_empty() {
                continue;
            }

            let exit_status = func(&**backend, packages, noconfirm).with_context(|| {
                format!("{verb_continuous} packages for {}", backend.get_section())
            })?;

            match exit_status.code() {
                Some(val) => ensure!(val == 0, "command returned with exit code {val}"),
                None => bail!("could not {verb} packages for {}", backend.get_section()),
            }
        }
        Ok(())
    }

    pub(crate) fn show(&self) -> Result<()> {
        let mut parts = vec![];

        for (backend, packages) in self.iter() {
            if packages.is_empty() {
                continue;
            }

            let mut segment = String::new();

            segment.write_str(&format!("[{}]", backend.get_section()))?;
            for package in packages {
                segment.write_str(&format!("\n{package}"))?;
            }

            parts.push(segment);
        }

        let mut output = String::new();
        let mut iter = parts.iter().peekable();

        while let Some(part) = iter.next() {
            output.write_str(part)?;
            if iter.peek().is_some() {
                output.write_str("\n\n")?;
            }
        }

        println!("{output}");

        Ok(())
    }
}

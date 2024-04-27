use std::fmt::Write;

use anyhow::{Context, Result};

use crate::prelude::*;

/// A vector of tuples containing a Backends and a vector of unmanaged packages
/// for that backend.
///
/// This struct is used to store a list of unmanaged packages or missing packages
/// for all backends.
#[derive(Debug)]
pub struct ToDoPerBackend(Vec<(AnyBackend, Packages)>);
impl ToDoPerBackend {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, item: (AnyBackend, Packages)) {
        self.0.push(item);
    }

    pub fn iter(&self) -> impl Iterator<Item = &(AnyBackend, Packages)> {
        self.0.iter()
    }

    pub fn nothing_to_do_for_all_backends(&self) -> bool {
        self.0.iter().all(|(_, diff)| diff.is_empty())
    }

    pub fn install_missing_packages(&self, noconfirm: bool) -> Result<()> {
        for (backend, packages) in &self.0 {
            if packages.is_empty() {
                continue;
            }

            backend
                .install_packages(packages, noconfirm)
                .with_context(|| format!("installing packages for {backend}"))?;
        }
        Ok(())
    }

    pub fn remove_unmanaged_packages(&self, noconfirm: bool) -> Result<()> {
        for (backend, packages) in &self.0 {
            if packages.is_empty() {
                continue;
            }

            backend
                .remove_packages(packages, noconfirm)
                .with_context(|| format!("removing packages for {backend}"))?;
        }
        Ok(())
    }

    pub fn show(&self) -> Result<()> {
        let mut parts = vec![];

        for (backend, packages) in self.iter() {
            if packages.is_empty() {
                continue;
            }

            let mut segment = String::new();

            segment.write_str(&format!("[{backend}]"))?;
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
impl Default for ToDoPerBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for ToDoPerBackend {
    type Item = (AnyBackend, Packages);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

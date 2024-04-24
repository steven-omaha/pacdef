use anyhow::Result;

use crate::prelude::*;

#[derive(Debug)]
pub struct Strategy {
    backend: AnyBackend,
    delete: Vec<Package>,
    as_dependency: Vec<Package>,
    assign_group: Vec<(Package, Group)>,
}

impl Strategy {
    pub fn new(
        backend: AnyBackend,
        delete: Vec<Package>,
        as_dependency: Vec<Package>,
        assign_group: Vec<(Package, Group)>,
    ) -> Self {
        Self {
            backend,
            delete,
            as_dependency,
            assign_group,
        }
    }

    pub fn execute(self) -> Result<()> {
        if !self.delete.is_empty() {
            self.backend.remove_packages(&self.delete, false)?;
        }

        if !self.as_dependency.is_empty() {
            self.backend.make_dependency(&self.as_dependency)?;
        }

        if !self.assign_group.is_empty() {
            self.backend.assign_group(self.assign_group)?;
        }

        Ok(())
    }

    pub fn show(&self) {
        if self.nothing_to_do() {
            return;
        }

        println!("[{}]", self.backend.backend_info().section);

        if !self.delete.is_empty() {
            println!("delete:");
            for p in &self.delete {
                println!("  {p}");
            }
        }

        if !self.as_dependency.is_empty() {
            println!("as dependency:");
            for p in &self.as_dependency {
                println!("  {p}");
            }
        }

        if !self.assign_group.is_empty() {
            println!("assign groups:");
            for (p, g) in &self.assign_group {
                println!("  {p} -> {}", g.name);
            }
        }
    }

    pub fn nothing_to_do(&self) -> bool {
        self.delete.is_empty() && self.as_dependency.is_empty() && self.assign_group.is_empty()
    }
}

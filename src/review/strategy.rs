use std::rc::Rc;

use anyhow::Result;

use crate::backend::Backend;
use crate::{Group, Package};

use super::datastructures::{ReviewAction, ReviewsPerBackend};

#[derive(Debug)]
pub(super) struct Strategy {
    backend: Box<dyn Backend>,
    delete: Vec<Package>,
    as_dependency: Vec<Package>,
    assign_group: Vec<(Package, Rc<Group>)>,
}

impl Strategy {
    pub(super) fn new(
        backend: Box<dyn Backend>,
        delete: Vec<Package>,
        as_dependency: Vec<Package>,
        assign_group: Vec<(Package, Rc<Group>)>,
    ) -> Self {
        Self {
            backend,
            delete,
            as_dependency,
            assign_group,
        }
    }

    pub(super) fn execute(self) -> Result<()> {
        if !self.delete.is_empty() {
            self.backend.remove_packages(&self.delete)?;
        }

        if !self.as_dependency.is_empty() {
            self.backend.make_dependency(&self.as_dependency)?;
        }

        if !self.assign_group.is_empty() {
            self.backend.assign_group(self.assign_group);
        }

        Ok(())
    }

    pub(super) fn show(&self) {
        if self.nothing_to_do() {
            return;
        }

        println!("[{}]", self.backend.get_section());

        if !self.delete.is_empty() {
            println!("delete:");
            for p in &self.delete {
                println!("  {p}");
            }
        }

        if !self.as_dependency.is_empty() {
            println!("as depdendency:");
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

    pub(super) fn nothing_to_do(&self) -> bool {
        self.delete.is_empty() && self.as_dependency.is_empty() && self.assign_group.is_empty()
    }
}

impl From<ReviewsPerBackend> for Vec<Strategy> {
    fn from(reviews: ReviewsPerBackend) -> Self {
        let mut result = vec![];

        for (backend, actions) in reviews.0 {
            let (to_delete, assign_group, as_dependency) = divide_actions(actions);

            result.push(Strategy::new(
                backend,
                to_delete,
                as_dependency,
                assign_group,
            ));
        }

        result
    }
}

fn divide_actions(
    actions: Vec<ReviewAction>,
) -> (Vec<Package>, Vec<(Package, Rc<Group>)>, Vec<Package>) {
    let mut to_delete = vec![];
    let mut assign_group = vec![];
    let mut as_dependency = vec![];

    for action in actions {
        match action {
            ReviewAction::Delete(package) => to_delete.push(package),
            ReviewAction::AssignGroup(package, group) => assign_group.push((package, group)),
            ReviewAction::AsDependency(package) => as_dependency.push(package),
        }
    }

    (to_delete, assign_group, as_dependency)
}

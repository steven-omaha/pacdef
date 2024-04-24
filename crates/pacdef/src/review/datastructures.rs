use crate::prelude::*;

use super::strategy::Strategy;

#[derive(Debug, PartialEq)]
pub enum ReviewAction {
    AsDependency(Package),
    Delete(Package),
    AssignGroup(Package, Group),
}

#[derive(Debug)]
pub enum ReviewIntention {
    AsDependency,
    AssignGroup,
    Delete,
    Info,
    Invalid,
    Skip,
    Quit,
    Apply,
}

#[derive(Debug)]
pub struct ReviewsPerBackend {
    items: Vec<(AnyBackend, Vec<ReviewAction>)>,
}

impl ReviewsPerBackend {
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    pub fn nothing_to_do(&self) -> bool {
        self.items.iter().all(|(_, vec)| vec.is_empty())
    }

    pub fn push(&mut self, value: (AnyBackend, Vec<ReviewAction>)) {
        self.items.push(value);
    }

    /// Convert the reviews per backend to a vector of [`Strategy`], where one `Strategy` contains
    /// all actions that must be executed for a [`Backend`].
    ///
    /// If there are no actions for a `Backend`, then that `Backend` is removed from the return
    /// value.
    pub fn into_strategies(self) -> Vec<Strategy> {
        let mut result = vec![];

        for (backend, actions) in self {
            let mut to_delete = vec![];
            let mut assign_group = vec![];
            let mut as_dependency = vec![];

            extract_actions(
                actions,
                &mut to_delete,
                &mut assign_group,
                &mut as_dependency,
            );

            result.push(Strategy::new(
                backend,
                to_delete,
                as_dependency,
                assign_group,
            ));
        }

        result.retain(|s| !s.nothing_to_do());

        result
    }
}

impl IntoIterator for ReviewsPerBackend {
    type Item = (AnyBackend, Vec<ReviewAction>);

    type IntoIter = std::vec::IntoIter<(AnyBackend, Vec<ReviewAction>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

pub enum ContinueWithReview {
    Yes,
    No,
    NoAndApply,
}

fn extract_actions(
    actions: Vec<ReviewAction>,
    to_delete: &mut Vec<Package>,
    assign_group: &mut Vec<(Package, Group)>,
    as_dependency: &mut Vec<Package>,
) {
    for action in actions {
        match action {
            ReviewAction::Delete(package) => to_delete.push(package),
            ReviewAction::AssignGroup(package, group) => assign_group.push((package, group)),
            ReviewAction::AsDependency(package) => as_dependency.push(package),
        }
    }
}

use std::rc::Rc;

use crate::backend::Backend;
use crate::{Group, Package};

#[derive(Debug, PartialEq)]
pub(super) enum ReviewAction {
    AsDependency(Package),
    Delete(Package),
    AssignGroup(Package, Rc<Group>),
}

#[derive(Debug)]
pub(super) enum ReviewIntention {
    AsDependency,
    AssignGroup,
    Delete,
    Info,
    Invalid,
    Skip,
    Quit,
}

#[derive(Debug)]
pub(super) struct ReviewsPerBackend {
    items: Vec<(Box<dyn Backend>, Vec<ReviewAction>)>,
}

impl ReviewsPerBackend {
    pub(super) fn new() -> Self {
        Self { items: vec![] }
    }

    pub(super) fn nothing_to_do(&self) -> bool {
        self.items.iter().all(|(_, vec)| vec.is_empty())
    }

    pub(super) fn push(&mut self, value: (Box<dyn Backend>, Vec<ReviewAction>)) {
        self.items.push(value);
    }
}

impl IntoIterator for ReviewsPerBackend {
    type Item = (Box<dyn Backend>, Vec<ReviewAction>);

    type IntoIter = std::vec::IntoIter<(Box<dyn Backend>, Vec<ReviewAction>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

pub(super) enum ContinueWithReview {
    Yes,
    No,
}

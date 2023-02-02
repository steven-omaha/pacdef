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
pub(super) struct ReviewsPerBackend(pub Vec<(Box<dyn Backend>, Vec<ReviewAction>)>);

impl ReviewsPerBackend {
    pub(super) fn new() -> Self {
        Self(vec![])
    }

    pub(super) fn nothing_to_do(&self) -> bool {
        self.0.iter().all(|(_, vec)| vec.is_empty())
    }
}

pub(super) enum ContinueWithReview {
    Yes,
    No,
}

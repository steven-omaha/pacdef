use super::{Backend, Backends};

#[derive(Debug)]
pub(crate) struct BackendIter {
    pub(crate) next: Option<Backends>,
}

impl Iterator for BackendIter {
    type Item = Box<dyn Backend>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.next {
            None => None,
            Some(b) => {
                let result = b.get_backend();
                self.next = b.next();
                Some(result)
            }
        }
    }
}

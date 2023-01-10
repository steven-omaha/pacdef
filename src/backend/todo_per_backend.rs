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

    pub(crate) fn into_iter(self) -> impl Iterator<Item = (Box<dyn Backend>, Vec<Package>)> {
        self.0.into_iter()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &(Box<dyn Backend>, Vec<Package>)> {
        self.0.iter()
    }

    pub(crate) fn nothing_to_do_for_all_backends(&self) -> bool {
        self.0.iter().all(|(_, diff)| diff.is_empty())
    }

    pub(crate) fn install_missing_packages(&self) {
        self.0
            .iter()
            .for_each(|(backend, diff)| backend.install_packages(diff));
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.iter().all(|(_, packages)| packages.is_empty())
    }

    pub(crate) fn show(&self) {
        for (backend, packages) in self.iter() {
            if packages.is_empty() {
                continue;
            }
            println!("{}", backend.get_section());
            for package in packages {
                println!("  {package}");
            }
        }
    }
}

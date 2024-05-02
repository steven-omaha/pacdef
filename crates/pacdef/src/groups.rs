use crate::prelude::*;
use anyhow::Result;

use std::collections::BTreeMap;

pub struct Groups {
    pub groups: BTreeMap<String, PackagesInstall>,
}
impl Groups {
    pub fn to_packages_install(&self) -> PackagesInstall {
        let mut packages = PackagesInstall::default();

        for group in self.groups.values() {
            packages.append(&mut group.clone());
        }

        packages
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    pub fn load(_: &Config) -> Result<Self> {
        todo!()
    }

}

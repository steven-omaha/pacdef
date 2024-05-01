use std::collections::BTreeMap;

use crate::prelude::*;

pub type Groups = BTreeMap<String, Group>;

#[derive(Debug, Clone)]
pub struct Group {
    pub apt: BTreeMap<<Apt as Backend>::PackageId, <Apt as Backend>::InstallOptions>,
    pub arch: BTreeMap<<Arch as Backend>::PackageId, <Arch as Backend>::InstallOptions>,
    pub cargo: BTreeMap<<Cargo as Backend>::PackageId, <Cargo as Backend>::InstallOptions>,
    pub dnf: BTreeMap<<Dnf as Backend>::PackageId, <Dnf as Backend>::InstallOptions>,
    pub flatpack: BTreeMap<<Flatpak as Backend>::PackageId, <Flatpak as Backend>::InstallOptions>,
    pub pip: BTreeMap<<Pip as Backend>::PackageId, <Pip as Backend>::InstallOptions>,
    pub pipx: BTreeMap<<Pipx as Backend>::PackageId, <Pipx as Backend>::InstallOptions>,
    pub rustup: BTreeMap<<Rustup as Backend>::PackageId, <Rustup as Backend>::InstallOptions>,
    pub xbps: BTreeMap<<Xbps as Backend>::PackageId, <Xbps as Backend>::InstallOptions>,
}

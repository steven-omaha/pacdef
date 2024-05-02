use anyhow::Result;
use std::collections::BTreeMap;

use crate::prelude::*;

pub struct Groups {
    groups: BTreeMap<String, BackendPackages>,
}
impl Groups {
    pub fn to_backend_packages(mut self) -> BackendPackages {
        let mut backend_packages = BackendPackages::new();

        for group in self.groups.values_mut() {
            backend_packages.append(group);
        }

        backend_packages
    }
}

#[derive(Debug, Clone)]
pub struct BackendPackages {
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
impl BackendPackages {
    pub fn new() -> Self {
        BackendPackages {
            apt: BTreeMap::new(),
            arch: BTreeMap::new(),
            cargo: BTreeMap::new(),
            dnf: BTreeMap::new(),
            flatpack: BTreeMap::new(),
            pip: BTreeMap::new(),
            pipx: BTreeMap::new(),
            rustup: BTreeMap::new(),
            xbps: BTreeMap::new(),
        }
    }
    pub fn append(&mut self, other: &mut Self) {
        self.apt.append(&mut other.apt);
        self.arch.append(&mut other.arch);
        self.cargo.append(&mut other.cargo);
        self.dnf.append(&mut other.dnf);
        self.flatpack.append(&mut other.flatpack);
        self.pip.append(&mut other.pip);
        self.pipx.append(&mut other.pipx);
        self.rustup.append(&mut other.rustup);
        self.xbps.append(&mut other.xbps);
    }
    pub fn install(self, no_confirm: bool, config: &Config) -> Result<()> {
        let apt = Apt::install_packages(&self.apt, no_confirm, config);
        let arch = Arch::install_packages(&self.arch, no_confirm, config);
        let cargo = Cargo::install_packages(&self.cargo, no_confirm, config);
        let dnf = Dnf::install_packages(&self.dnf, no_confirm, config);
        let flatpak = Flatpak::install_packages(&self.flatpack, no_confirm, config);
        let pip = Pip::install_packages(&self.pip, no_confirm, config);
        let pipx = Pipx::install_packages(&self.pipx, no_confirm, config);
        let rustup = Rustup::install_packages(&self.rustup, no_confirm, config);
        let xbps = Xbps::install_packages(&self.xbps, no_confirm, config);

        apt.and(arch)
            .and(cargo)
            .and(dnf)
            .and(flatpak)
            .and(pip)
            .and(pipx)
            .and(rustup)
            .and(xbps)
    }
}

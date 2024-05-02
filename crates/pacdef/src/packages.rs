use anyhow::Result;
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use crate::prelude::*;

pub struct Groups {
    groups: BTreeMap<String, PackagesInstall>,
}
impl Groups {
    pub fn to_packages_install(&self) -> PackagesInstall {
        let mut packages = PackagesInstall::default();

        for group in self.groups.values() {
            packages.append(&mut group.clone());
        }

        packages
    }
}

#[derive(Debug, Clone, Default)]
pub struct PackagesIds {
    pub apt: BTreeSet<<Apt as Backend>::PackageId>,
    pub arch: BTreeSet<<Arch as Backend>::PackageId>,
    pub cargo: BTreeSet<<Cargo as Backend>::PackageId>,
    pub dnf: BTreeSet<<Dnf as Backend>::PackageId>,
    pub flatpak: BTreeSet<<Flatpak as Backend>::PackageId>,
    pub pip: BTreeSet<<Pip as Backend>::PackageId>,
    pub pipx: BTreeSet<<Pipx as Backend>::PackageId>,
    pub rustup: BTreeSet<<Rustup as Backend>::PackageId>,
    pub xbps: BTreeSet<<Xbps as Backend>::PackageId>,
}
#[derive(Debug, Clone, Default)]
pub struct PackagesInstall {
    pub apt: BTreeMap<<Apt as Backend>::PackageId, <Apt as Backend>::InstallOptions>,
    pub arch: BTreeMap<<Arch as Backend>::PackageId, <Arch as Backend>::InstallOptions>,
    pub cargo: BTreeMap<<Cargo as Backend>::PackageId, <Cargo as Backend>::InstallOptions>,
    pub dnf: BTreeMap<<Dnf as Backend>::PackageId, <Dnf as Backend>::InstallOptions>,
    pub flatpak: BTreeMap<<Flatpak as Backend>::PackageId, <Flatpak as Backend>::InstallOptions>,
    pub pip: BTreeMap<<Pip as Backend>::PackageId, <Pip as Backend>::InstallOptions>,
    pub pipx: BTreeMap<<Pipx as Backend>::PackageId, <Pipx as Backend>::InstallOptions>,
    pub rustup: BTreeMap<<Rustup as Backend>::PackageId, <Rustup as Backend>::InstallOptions>,
    pub xbps: BTreeMap<<Xbps as Backend>::PackageId, <Xbps as Backend>::InstallOptions>,
}
#[derive(Debug, Clone, Default)]
pub struct PackagesRemove {
    pub apt: BTreeMap<<Apt as Backend>::PackageId, <Apt as Backend>::RemoveOptions>,
    pub arch: BTreeMap<<Arch as Backend>::PackageId, <Arch as Backend>::RemoveOptions>,
    pub cargo: BTreeMap<<Cargo as Backend>::PackageId, <Cargo as Backend>::RemoveOptions>,
    pub dnf: BTreeMap<<Dnf as Backend>::PackageId, <Dnf as Backend>::RemoveOptions>,
    pub flatpak: BTreeMap<<Flatpak as Backend>::PackageId, <Flatpak as Backend>::RemoveOptions>,
    pub pip: BTreeMap<<Pip as Backend>::PackageId, <Pip as Backend>::RemoveOptions>,
    pub pipx: BTreeMap<<Pipx as Backend>::PackageId, <Pipx as Backend>::RemoveOptions>,
    pub rustup: BTreeMap<<Rustup as Backend>::PackageId, <Rustup as Backend>::RemoveOptions>,
    pub xbps: BTreeMap<<Xbps as Backend>::PackageId, <Xbps as Backend>::RemoveOptions>,
}
#[derive(Debug, Clone, Default)]
pub struct PackagesQuery {
    pub apt: BTreeMap<<Apt as Backend>::PackageId, <Apt as Backend>::QueryInfo>,
    pub arch: BTreeMap<<Arch as Backend>::PackageId, <Arch as Backend>::QueryInfo>,
    pub cargo: BTreeMap<<Cargo as Backend>::PackageId, <Cargo as Backend>::QueryInfo>,
    pub dnf: BTreeMap<<Dnf as Backend>::PackageId, <Dnf as Backend>::QueryInfo>,
    pub flatpak: BTreeMap<<Flatpak as Backend>::PackageId, <Flatpak as Backend>::QueryInfo>,
    pub pip: BTreeMap<<Pip as Backend>::PackageId, <Pip as Backend>::QueryInfo>,
    pub pipx: BTreeMap<<Pipx as Backend>::PackageId, <Pipx as Backend>::QueryInfo>,
    pub rustup: BTreeMap<<Rustup as Backend>::PackageId, <Rustup as Backend>::QueryInfo>,
    pub xbps: BTreeMap<<Xbps as Backend>::PackageId, <Xbps as Backend>::QueryInfo>,
}

macro_rules! impl_append {
    () => {
        pub fn append(&mut self, other: &mut Self) {
            self.apt.append(&mut other.apt);
            self.arch.append(&mut other.arch);
            self.cargo.append(&mut other.cargo);
            self.dnf.append(&mut other.dnf);
            self.flatpak.append(&mut other.flatpak);
            self.pip.append(&mut other.pip);
            self.pipx.append(&mut other.pipx);
            self.rustup.append(&mut other.rustup);
            self.xbps.append(&mut other.xbps);
        }
    };
}
macro_rules! impl_to_packages_ids {
    () => {
        pub fn to_packages_ids(self) -> PackagesIds {
            PackagesIds {
                apt: self.apt.into_keys().collect(),
                arch: self.arch.into_keys().collect(),
                cargo: self.cargo.into_keys().collect(),
                dnf: self.dnf.into_keys().collect(),
                flatpak: self.flatpak.into_keys().collect(),
                pip: self.pip.into_keys().collect(),
                pipx: self.pipx.into_keys().collect(),
                rustup: self.rustup.into_keys().collect(),
                xbps: self.xbps.into_keys().collect(),
            }
        }
    };
}
impl PackagesIds {
    impl_append!();
}
impl PackagesInstall {
    impl_append!();
    impl_to_packages_ids!();
}
impl PackagesRemove {
    impl_append!();
    impl_to_packages_ids!();
}
impl PackagesQuery {
    impl_append!();
    impl_to_packages_ids!();
}

impl PackagesIds {
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            apt: self.apt.difference(&other.apt).cloned().collect(),
            arch: self.arch.difference(&other.arch).cloned().collect(),
            cargo: self.cargo.difference(&other.cargo).cloned().collect(),
            dnf: self.dnf.difference(&other.dnf).cloned().collect(),
            flatpak: self.flatpak.difference(&other.flatpak).cloned().collect(),
            pip: self.pip.difference(&other.pip).cloned().collect(),
            pipx: self.pipx.difference(&other.pipx).cloned().collect(),
            rustup: self.rustup.difference(&other.rustup).cloned().collect(),
            xbps: self.xbps.difference(&other.xbps).cloned().collect(),
        }
    }
}

impl PackagesInstall {
    pub fn install(self, no_confirm: bool, config: &Config) -> Result<()> {
        let apt = Apt::install_packages(&self.apt, no_confirm, config);
        let arch = Arch::install_packages(&self.arch, no_confirm, config);
        let cargo = Cargo::install_packages(&self.cargo, no_confirm, config);
        let dnf = Dnf::install_packages(&self.dnf, no_confirm, config);
        let flatpak = Flatpak::install_packages(&self.flatpak, no_confirm, config);
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

impl PackagesQuery {
    pub fn installed(config: &Config) -> Result<Self> {
        let apt = Apt::query_installed_packages(config)?;
        let arch = Arch::query_installed_packages(config)?;
        let cargo = Cargo::query_installed_packages(config)?;
        let dnf = Dnf::query_installed_packages(config)?;
        let flatpak = Flatpak::query_installed_packages(config)?;
        let pip = Pip::query_installed_packages(config)?;
        let pipx = Pipx::query_installed_packages(config)?;
        let rustup = Rustup::query_installed_packages(config)?;
        let xbps = Xbps::query_installed_packages(config)?;

        Ok(Self {
            apt,
            arch,
            cargo,
            dnf,
            flatpak,
            pip,
            pipx,
            rustup,
            xbps,
        })
    }
}

impl PackagesIds {
    //todo this could be improved by making the config for disabled_backends more strongly typed
    pub fn clear_backends(&mut self, backend_names: &Vec<String>) {
        for backend_name in backend_names {
            let backend = match AnyBackend::from_str(&backend_name) {
                Ok(x) => x,
                Err(e) => {
                    log::warn!("{e}");
                    continue;
                }
            };

            match backend {
                AnyBackend::Apt(_) => self.apt.clear(),
                AnyBackend::Cargo(_) => self.cargo.clear(),
                AnyBackend::Dnf(_) => self.dnf.clear(),
                AnyBackend::Flatpak(_) => self.flatpak.clear(),
                AnyBackend::Pip(_) => self.pip.clear(),
                AnyBackend::Pipx(_) => self.pipx.clear(),
                AnyBackend::Rustup(_) => self.rustup.clear(),
                AnyBackend::Xbps(_) => self.xbps.clear(),
            }
        }
    }
    pub fn missing(groups: &Groups, config: &Config) -> Result<Self> {
        let requested = groups.to_packages_install();

        let installed = PackagesQuery::installed(config)?;

        let mut missing = requested
            .to_packages_ids()
            .difference(&installed.to_packages_ids());

        missing.clear_backends(&config.disabled_backends);

        Ok(missing)
    }
}

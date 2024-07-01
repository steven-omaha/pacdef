use anyhow::Result;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    str::FromStr,
};

use crate::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct PackagesIds {
    // pub apt: BTreeSet<<Apt as Backend>::PackageId>,
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
    // pub apt: BTreeMap<<Apt as Backend>::PackageId, <Apt as Backend>::InstallOptions>,
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
    // pub apt: BTreeMap<<Apt as Backend>::PackageId, <Apt as Backend>::RemoveOptions>,
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
    // pub apt: BTreeMap<<Apt as Backend>::PackageId, <Apt as Backend>::QueryInfo>,
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
        #[allow(dead_code)]
        pub fn append(&mut self, other: &mut Self) {
            // self.apt.append(&mut other.apt);
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
macro_rules! impl_into_packages_ids {
    () => {
        #[allow(dead_code)]
        pub fn into_packages_ids(self) -> PackagesIds {
            PackagesIds {
                // apt: self.apt.into_keys().collect(),
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
macro_rules! impl_is_empty {
    () => {
        #[allow(dead_code)]
        pub fn is_empty(&self) -> bool {
            // self.apt.is_empty()
            self.arch.is_empty()
                && self.cargo.is_empty()
                && self.dnf.is_empty()
                && self.flatpak.is_empty()
                && self.pip.is_empty()
                && self.pipx.is_empty()
                && self.rustup.is_empty()
                && self.xbps.is_empty()
        }
    };
}
impl PackagesIds {
    impl_append!();
    impl_is_empty!();
}
impl PackagesInstall {
    impl_append!();
    impl_is_empty!();
    impl_into_packages_ids!();
}
impl PackagesRemove {
    impl_append!();
    impl_is_empty!();
    impl_into_packages_ids!();
}
impl PackagesQuery {
    impl_append!();
    impl_is_empty!();
    impl_into_packages_ids!();
}

impl PackagesIds {
    pub fn difference(&self, other: &Self) -> Self {
        Self {
            // apt: self.apt.difference(&other.apt).cloned().collect(),
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
        // let apt = Apt::install_packages(&self.apt, no_confirm, config);
        let arch = Arch::install_packages(&self.arch, no_confirm, config);
        let cargo = Cargo::install_packages(&self.cargo, no_confirm, config);
        let dnf = Dnf::install_packages(&self.dnf, no_confirm, config);
        let flatpak = Flatpak::install_packages(&self.flatpak, no_confirm, config);
        let pip = Pip::install_packages(&self.pip, no_confirm, config);
        let pipx = Pipx::install_packages(&self.pipx, no_confirm, config);
        let rustup = Rustup::install_packages(&self.rustup, no_confirm, config);
        let xbps = Xbps::install_packages(&self.xbps, no_confirm, config);

        // apt.and(arch)
        arch.and(cargo)
            .and(dnf)
            .and(flatpak)
            .and(pip)
            .and(pipx)
            .and(rustup)
            .and(xbps)
    }

    #[rustfmt::skip]
    pub fn from_packages_ids_defaults(packages_ids: &PackagesIds) -> Self {
        Self {
            // apt: packages_ids.apt.iter().map(|x| (x.clone(), <Apt as Backend>::InstallOptions::default())).collect(),
            arch: packages_ids.arch.iter().map(|x| (x.clone(), <Arch as Backend>::InstallOptions::default())).collect(),
            cargo: packages_ids.cargo.iter().map(|x| (x.clone(), <Cargo as Backend>::InstallOptions::default())).collect(),
            dnf: packages_ids.dnf.iter().map(|x| (x.clone(), <Dnf as Backend>::InstallOptions::default())).collect(),
            flatpak: packages_ids.flatpak.iter().map(|x| (x.clone(), <Flatpak as Backend>::InstallOptions::default())).collect(),
            pip: packages_ids.pip.iter().map(|x| (x.clone(), <Pip as Backend>::InstallOptions::default())).collect(),
            pipx: packages_ids.pipx.iter().map(|x| (x.clone(), <Pipx as Backend>::InstallOptions::default())).collect(),
            rustup: packages_ids.rustup.iter().map(|x| (x.clone(), <Rustup as Backend>::InstallOptions::default())).collect(),
            xbps: packages_ids.xbps.iter().map(|x| (x.clone(), <Xbps as Backend>::InstallOptions::default())).collect(),
        }
    }
}

impl PackagesRemove {
    pub fn remove(self, no_confirm: bool, config: &Config) -> Result<()> {
        // let apt = Apt::remove_packages(&self.apt, no_confirm, config);
        let arch = Arch::remove_packages(&self.arch, no_confirm, config);
        let cargo = Cargo::remove_packages(&self.cargo, no_confirm, config);
        let dnf = Dnf::remove_packages(&self.dnf, no_confirm, config);
        let flatpak = Flatpak::remove_packages(&self.flatpak, no_confirm, config);
        let pip = Pip::remove_packages(&self.pip, no_confirm, config);
        let pipx = Pipx::remove_packages(&self.pipx, no_confirm, config);
        let rustup = Rustup::remove_packages(&self.rustup, no_confirm, config);
        let xbps = Xbps::remove_packages(&self.xbps, no_confirm, config);

        // apt.and(arch)
        arch.and(cargo)
            .and(dnf)
            .and(flatpak)
            .and(pip)
            .and(pipx)
            .and(rustup)
            .and(xbps)
    }

    #[rustfmt::skip]
    pub fn from_packages_ids_defaults(packages_ids: &PackagesIds) -> Self {
        Self {
            // apt: packages_ids.apt.iter().map(|x| (x.clone(), <Apt as Backend>::RemoveOptions::default())).collect(),
            arch: packages_ids.arch.iter().map(|x| (x.clone(), <Arch as Backend>::RemoveOptions::default())).collect(),
            cargo: packages_ids.cargo.iter().map(|x| (x.clone(), <Cargo as Backend>::RemoveOptions::default())).collect(),
            dnf: packages_ids.dnf.iter().map(|x| (x.clone(), <Dnf as Backend>::RemoveOptions::default())).collect(),
            flatpak: packages_ids.flatpak.iter().map(|x| (x.clone(), <Flatpak as Backend>::RemoveOptions::default())).collect(),
            pip: packages_ids.pip.iter().map(|x| (x.clone(), <Pip as Backend>::RemoveOptions::default())).collect(),
            pipx: packages_ids.pipx.iter().map(|x| (x.clone(), <Pipx as Backend>::RemoveOptions::default())).collect(),
            rustup: packages_ids.rustup.iter().map(|x| (x.clone(), <Rustup as Backend>::RemoveOptions::default())).collect(),
            xbps: packages_ids.xbps.iter().map(|x| (x.clone(), <Xbps as Backend>::RemoveOptions::default())).collect(),
        }
    }
}

impl PackagesQuery {
    pub fn installed(config: &Config) -> Result<Self> {
        // let apt = Apt::query_installed_packages(config)?;
        let arch = Arch::query_installed_packages(config)?;
        let cargo = Cargo::query_installed_packages(config)?;
        let dnf = Dnf::query_installed_packages(config)?;
        let flatpak = Flatpak::query_installed_packages(config)?;
        let pip = Pip::query_installed_packages(config)?;
        let pipx = Pipx::query_installed_packages(config)?;
        let rustup = Rustup::query_installed_packages(config)?;
        let xbps = Xbps::query_installed_packages(config)?;

        Ok(Self {
            // apt,
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
            let backend = match AnyBackend::from_str(backend_name) {
                Ok(x) => x,
                Err(e) => {
                    log::warn!("{e}");
                    continue;
                }
            };

            match backend {
                // AnyBackend::Apt(_) => self.apt.clear(),
                AnyBackend::Arch(_) => self.arch.clear(),
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
            .into_packages_ids()
            .difference(&installed.into_packages_ids());

        missing.clear_backends(&config.disabled_backends);

        Ok(missing)
    }
    pub fn unmanaged(groups: &Groups, config: &Config) -> Result<Self> {
        let requested = groups.to_packages_install();

        let installed = PackagesQuery::installed(config)?;

        let mut missing = installed
            .into_packages_ids()
            .difference(&requested.into_packages_ids());

        missing.clear_backends(&config.disabled_backends);

        Ok(missing)
    }
}

impl Display for PackagesIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        macro_rules! list {
            ($id:ident) => {
                let $id = itertools::Itertools::intersperse(
                    self.$id.iter().map(|x| x.to_string()),
                    "\n".to_string(),
                )
                .collect::<String>();
            };
        }

        // list!(apt);
        list!(cargo);
        list!(dnf);
        list!(flatpak);
        list!(pip);
        list!(pipx);
        list!(rustup);
        list!(xbps);

        write!(
            f,
            "
[cargo]
{cargo}

[dnf]
{dnf}

[flatpak]
{flatpak}

[pip]
{pip}

[pipx]
{pipx}

[rustup]
{rustup}

[xbps]
{xbps}
"
        )
    }
}

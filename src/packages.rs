use anyhow::Result;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    str::FromStr,
};

use crate::prelude::*;

macro_rules! generate_packages_ids {
    ($($name:ident: $backend:ident),*) => {
        #[derive(Debug, Clone, Default)]
        pub struct PackagesIds {
        $(
            pub $name: BTreeSet<<$backend as Backend>::PackageId>,
        )*
        }
    };
}

macro_rules! generate_packages_install {
    ($($name:ident: $backend:ident),*) => {
        #[derive(Debug, Clone, Default)]
        pub struct PackagesInstall {
        $(
            pub $name: BTreeMap<<$backend as Backend>::PackageId, <$backend as Backend>::InstallOptions>,
        )*
        }
    };
}

macro_rules! generate_packages_remove {
    ($($name:ident: $backend:ident),* ) => {
        #[derive(Debug, Clone, Default)]
        pub struct PackagesRemove {
        $(
            pub $name: BTreeMap<<$backend as Backend>::PackageId, <$backend as Backend>::RemoveOptions>,
        )*
        }
    };
}

macro_rules! generate_packages_query {
    ($($name:ident: $backend:ident),*) => {
        #[derive(Debug, Clone, Default)]
        pub struct PackagesQuery {
        $(
            pub $name: BTreeMap<<$backend as Backend>::PackageId, <$backend as Backend>::QueryInfo>,
        )*
        }
    };
}

macro_rules! append {
    ($($name:ident)*) => {
        pub fn append(&mut self, other: &mut Self) {
            $(
                self.$name.append(&mut other.$name);
            )*
        }
    };
}

macro_rules! is_empty {
    ($($name:ident)*) => {
        pub fn is_empty(&self) ->bool {
            $(
                self.$name.is_empty() &&
            )* true
        }
    };
}

macro_rules! impl_packages_ids {
    ($($name:ident: $backend:ident),*) => {
        impl PackagesIds {
            pub fn new() -> Self {
                Self {
                    $(
                        $name: BTreeSet::new(),
                    )*
                }
            }

            append!($($name)*);
            is_empty!($($name)*);

            pub fn difference(&self, other: &Self) -> Self {
                Self {
                    $(
                        $name: self.$name.difference(&other.$name).cloned().collect(),
                    )*
                }
            }

            pub fn insert_backend_package(&mut self, backend: AnyBackend, package: String) -> Result<()> {
                match backend {
                    $(
                      AnyBackend::$backend(_) => self.$name.insert(package.try_into()?),
                    )*
                };
                Ok(())
            }

            //todo this could be improved by making the config for disabled_backends more strongly typed
            //but if we make it more strongly typed we'll probably lose out the macro neatness
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
                        $(
                            AnyBackend::$backend(_) => self.$name.clear(),
                        )*
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

                let mut unmanaged = installed
                    .into_packages_ids()
                    .difference(&requested.into_packages_ids());

                unmanaged.clear_backends(&config.disabled_backends);

                Ok(unmanaged)
            }
        }
    };
}

macro_rules! into_packages_ids {
    ($($name:ident)*) => {
        pub fn into_packages_ids(self) -> PackagesIds {
            PackagesIds {
                $(
                    $name: self.$name.into_keys().collect(),
                )*
            }
        }
    };
}

macro_rules! impl_packages_installs {
    ($($name:ident: $backend:ident),*) => {
        impl PackagesInstall {
            append!($($name)*);
            is_empty!($($name)*);
            into_packages_ids!($($name)*);

            pub fn install(self, no_confirm: bool, config: &Config) -> Result<()> {
                $(
                    let $name = $backend::install_packages(&self.$name, no_confirm, config);
                )*
                Ok(())$(
                  .and($name)
                )*
            }

            pub fn from_packages_ids_defaults(packages_ids: &PackagesIds) -> Self {
                Self {
                    $(
                        $name: packages_ids.$name.iter().map(|x| (x.clone(), <$backend as Backend>::InstallOptions::default())).collect(),
                    )*
                }
            }


        }
    };
}

macro_rules! impl_packages_remove {
    ($($name:ident: $backend:ident),*) => {
        impl PackagesRemove {
            append!($($name)*);
            is_empty!($($name)*);
            into_packages_ids!($($name)*);

            pub fn remove(self, no_confirm: bool, config: &Config) -> Result<()> {
                $(
                    let $name = $backend::remove_packages(&self.$name, no_confirm, config);
                )*

                Ok(())$(
                  .and($name)
                )*
            }

            pub fn from_packages_ids_defaults(packages_ids: &PackagesIds) -> Self {
                Self {
                    $(
                        $name: packages_ids.$name.iter().map(|x| (x.clone(), <$backend as Backend>::RemoveOptions::default())).collect(),
                    )*
                }
            }
        }
    };
}

macro_rules! impl_packages_query {
    ($($name:ident:$backend:ident),*) => {
        impl PackagesQuery {
            append!($($name)*);
            is_empty!($($name)*);
            into_packages_ids!($($name)*);
            pub fn installed(config: &Config) -> Result<Self> {
                $(
                    let $name = $backend::query_installed_packages(config)?;
                )*

                Ok(Self {
                    $(
                        $name,
                    )*
                })
            }
        }
    };
}

macro_rules! impl_display_for_packages_ids {
    ($($name:ident),*) => {
        impl Display for PackagesIds {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                macro_rules! list {
                    ($id:ident) => {
                        let $id: String = itertools::Itertools::intersperse(
                            self.$id.iter().map(|x| x.to_string()),
                            "\n".to_string(),
                        )
                        .collect();
                    };
                }

                $(
                    list!($name);
                )*

                write!(
                    f,
                    "{}", [$(
                        stringify!([$name]),
                        $name.as_str(),
                        " ",
                    )*].join("\n")
                )
            }
        }
    };
}

macro_rules! generate_structs {
    ($($name:ident: $backend:ident),*) => {
        generate_packages_ids!( $($name:$backend),* );
        generate_packages_install!( $($name:$backend),* );
        generate_packages_remove!( $($name:$backend),* );
        generate_packages_query!( $($name:$backend),* );
        impl_packages_ids!($($name:$backend),*);
        impl_packages_installs!($($name:$backend),*);
        impl_packages_remove!($($name:$backend),*);
        impl_packages_query!($($name:$backend),*);
        impl_display_for_packages_ids!($($name),*);
    };
}

generate_structs!(
    arch: Arch,
    cargo: Cargo,
    dnf: Dnf,
    flatpak: Flatpak,
    pip: Pip,
    pipx: Pipx,
    rustup: Rustup,
    xbps: Xbps
);

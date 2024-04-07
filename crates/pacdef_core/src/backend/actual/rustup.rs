use crate::backend::backend_trait::{Backend, Switches, Text};
use crate::backend::macros::impl_backend_constants;
use crate::cmd::run_external_command;
use crate::{Group, Package};
use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus};

const BINARY: Text = "rustup";
const SECTION: Text = "rustup";

const SWITCHES_INSTALL: Switches = &["component", "add"];
const SWITCHES_INFO: Switches = &["component", "list", "--installed"];
const SWITCHES_MAKE_DEPENDENCY: Switches = &[];
const SWITCHES_NOCONFIRM: Switches = &[];
const SWITCHES_REMOVE: Switches = &["component", "remove"];

const SUPPORTS_AS_DEPENDENCY: bool = false;

#[derive(Debug, Clone)]
pub struct Rustup {
    pub(crate) packages: HashSet<Package>,
}

#[derive(Debug)]
enum Repotype {
    Toolchain,
    Component,
}

impl Repotype {
    fn try_from<T>(value: T) -> Result<Self>
    where
        T: AsRef<str>,
    {
        let value = value.as_ref();
        let result = match value {
            "toolchain" => Self::Toolchain,
            "component" => Self::Component,
            _ => bail!("{} is neither toolchain nor component", value),
        };
        Ok(result)
    }
}

/// A package as used exclusively in the rustup backend. Contrary to other packages, this does not
/// have an (optional) repository and a name, but is either a component or a toolchain, has a
/// toolchain version, and if it is a toolchain also a name.
#[derive(Debug)]
struct RustupPackage {
    /// Whether it is a toolchain or a component.
    pub repotype: Repotype,
    /// The name of the toolchain this belongs to (stable, nightly, a pinned version)
    pub toolchain: String,
    /// If it is a toolchain, it will not have a component name.
    /// If it is a component, this will be its name.
    pub component: Option<String>,
}

impl RustupPackage {
    /// Creates a new [`RustupPackage`].
    ///
    /// # Panics
    ///
    /// Panics if
    /// - repotype is Toolchain and component is Some, or
    /// - repotype is Component and component is None.
    fn new(repotype: Repotype, toolchain: String, component: Option<String>) -> Self {
        match repotype {
            Repotype::Toolchain => assert!(component.is_none()),
            Repotype::Component => assert!(component.is_some()),
        };

        Self {
            repotype,
            toolchain,
            component,
        }
    }
}

impl TryFrom<&Package> for RustupPackage {
    type Error = anyhow::Error;

    fn try_from(package: &Package) -> Result<Self> {
        let repo = package.repo.as_ref().context("getting repo from package")?;
        let repotype = Repotype::try_from(repo).context("getting repotype")?;

        let (toolchain, component) = match repotype {
            Repotype::Toolchain => (package.name.to_string(), None),
            Repotype::Component => {
                let (toolchain, component) = package
                    .name
                    .split_once('/')
                    .context("splitting package into toolchain and component")?;
                (toolchain.to_string(), Some(component.into()))
            }
        };

        Ok(Self::new(repotype, toolchain, component))
    }
}

impl Backend for Rustup {
    impl_backend_constants!();

    fn get_all_installed_packages(&self) -> Result<HashSet<Package>> {
        let toolchains_vec = self
            .run_toolchain_command(get_info_switches(Repotype::Toolchain))
            .context("Getting installed toolchains")?;

        let toolchains: HashSet<Package> = toolchains_vec
            .iter()
            .map(|name| ["toolchain", name].join("/").into())
            .collect();

        let components: HashSet<Package> = self
            .run_component_command(get_info_switches(Repotype::Component), &toolchains_vec)
            .context("Getting installed components")?
            .iter()
            .map(|name| ["component", name].join("/").into())
            .collect();

        let mut packages = HashSet::new();

        packages.extend(toolchains);
        packages.extend(components);

        Ok(packages)
    }

    fn get_explicitly_installed_packages(&self) -> Result<HashSet<Package>> {
        self.get_all_installed_packages()
            .context("Getting all installed packages")
    }

    fn make_dependency(&self, _: &[Package]) -> Result<ExitStatus> {
        panic!("Not supported by {}", self.get_binary())
    }

    fn install_packages(&self, packages: &[Package], _: bool) -> Result<ExitStatus> {
        let packages = convert_all_packages_to_rustup_packages(packages)?;

        let (toolchains, components) = sort_packages_into_toolchains_and_components(packages);

        self.install_toolchains(toolchains)?;
        self.install_components(components)?;

        Ok(ExitStatus::from_raw(0))
    }

    fn remove_packages(&self, packages: &[Package], _: bool) -> Result<ExitStatus> {
        let rustup_packages = convert_all_packages_to_rustup_packages(packages)?;

        let (toolchains, components) =
            sort_packages_into_toolchains_and_components(rustup_packages);

        let mut removed_toolchains = vec![];

        if !toolchains.is_empty() {
            let mut cmd = Command::new(self.get_binary());
            cmd.args(get_remove_switches(Repotype::Toolchain));

            for toolchain_package in &toolchains {
                let name = toolchain_package.toolchain.as_str();
                cmd.arg(name);
                removed_toolchains.push(name);
            }

            run_external_command(cmd)
                .with_context(|| format!("removing toolchains [{toolchains:?}]"))?;
        }

        for component_package in components {
            let mut cmd = Command::new(self.get_binary());
            cmd.args(get_remove_switches(Repotype::Component));

            if toolchain_of_component_was_already_removed(&removed_toolchains, &component_package) {
                continue;
            }

            cmd.arg(&component_package.toolchain);
            cmd.arg(
                component_package
                    .component
                    .as_ref()
                    .expect("the constructor ensures this cannot be None"),
            );

            run_external_command(cmd)
                .with_context(|| format!("removing component {component_package:?}"))?;
        }
        Ok(ExitStatus::from_raw(0))
    }
}

fn convert_all_packages_to_rustup_packages(packages: &[Package]) -> Result<Vec<RustupPackage>> {
    let mut result = vec![];

    for package in packages {
        let rustup_package = RustupPackage::try_from(package).with_context(|| {
            format!(
                "converting pacdef package {} to rustup package",
                package.name
            )
        })?;
        result.push(rustup_package);
    }

    Ok(result)
}

fn toolchain_of_component_was_already_removed(
    removed_toolchains: &[&str],
    component: &RustupPackage,
) -> bool {
    removed_toolchains.contains(&component.toolchain.as_ref())
}

fn sort_packages_into_toolchains_and_components(
    packages: Vec<RustupPackage>,
) -> (Vec<RustupPackage>, Vec<RustupPackage>) {
    let mut toolchains = vec![];
    let mut components = vec![];

    for package in packages {
        match package.repotype {
            Repotype::Toolchain => toolchains.push(package),
            Repotype::Component => components.push(package),
        }
    }

    (toolchains, components)
}

impl Rustup {
    pub(crate) fn new() -> Self {
        Self {
            packages: HashSet::new(),
        }
    }

    fn run_component_command(&self, args: &[&str], toolchains: &[String]) -> Result<Vec<String>> {
        let mut val = Vec::new();

        for toolchain in toolchains {
            let mut cmd = Command::new(self.get_binary());
            cmd.args(args).arg(toolchain);

            let output = String::from_utf8(cmd.output()?.stdout)?;

            for component in output.lines() {
                install_components(component, toolchain, &mut val);
            }
        }

        Ok(val)
    }

    fn run_toolchain_command(&self, args: &[&str]) -> Result<Vec<String>> {
        let mut cmd = Command::new(self.get_binary());
        cmd.args(args);

        let output = String::from_utf8(cmd.output()?.stdout)?;

        let mut val = Vec::new();

        for line in output.lines() {
            let toolchain = line.split('-').next();
            match toolchain {
                Some(name) => val.push(name.to_string()),
                None => bail!("Toolchain name not provided!"),
            }
        }

        Ok(val)
    }

    fn install_toolchains(&self, toolchains: Vec<RustupPackage>) -> Result<()> {
        if toolchains.is_empty() {
            return Ok(());
        }
        let mut cmd = Command::new(self.get_binary());
        cmd.args(get_install_switches(Repotype::Toolchain));

        for toolchain in toolchains {
            cmd.arg(&toolchain.toolchain);
        }

        run_external_command(cmd).context("installing toolchains")?;

        Ok(())
    }

    fn install_components(&self, components: Vec<RustupPackage>) -> Result<()> {
        if components.is_empty() {
            return Ok(());
        }

        let components_by_toolchain = group_components_by_toolchains(components);

        for components_for_one_toolchain in components_by_toolchain {
            let mut cmd = Command::new(self.get_binary());
            cmd.args(get_install_switches(Repotype::Component));

            let the_toolchain = &components_for_one_toolchain
                .first()
                .expect("will have at least one element")
                .toolchain;

            cmd.arg(the_toolchain);

            for component_package in &components_for_one_toolchain {
                let actual_component = component_package
                    .component
                    .as_ref()
                    .expect("constructor makes sure this is Some");

                cmd.arg(actual_component);
            }

            run_external_command(cmd)
                .with_context(|| format!("installing [{components_for_one_toolchain:?}]"))?;
        }

        Ok(())
    }
}

fn get_install_switches(repotype: Repotype) -> Switches {
    match repotype {
        Repotype::Toolchain => &["toolchain", "install"],
        Repotype::Component => &["component", "add", "--toolchain"],
    }
}

fn get_remove_switches(repotype: Repotype) -> Switches {
    match repotype {
        Repotype::Toolchain => &["toolchain", "uninstall"],
        Repotype::Component => &["component", "remove", "--toolchain"],
    }
}

fn get_info_switches(repotype: Repotype) -> Switches {
    match repotype {
        Repotype::Toolchain => &["toolchain", "list"],
        Repotype::Component => &["component", "list", "--installed", "--toolchain"],
    }
}

fn install_components(line: &str, toolchain: &str, val: &mut Vec<String>) {
    let mut chunks = line.splitn(3, '-');
    let component = chunks.next().expect("Component name is empty!");
    match component {
        // these are the only components that have a single word name
        "cargo" | "rustfmt" | "clippy" | "miri" | "rls" | "rustc" => {
            val.push([toolchain, component].join("/"));
        }
        // all the others have two words hyphenated as component names
        _ => {
            let component = [
                component,
                chunks
                    .next()
                    .expect("No such component is managed by rustup"),
            ]
            .join("-");
            val.push([toolchain, component.as_str()].join("/"));
        }
    }
}

fn group_components_by_toolchains(components: Vec<RustupPackage>) -> Vec<Vec<RustupPackage>> {
    let mut result = vec![];

    let mut toolchains: Vec<String> = vec![];

    for component in components {
        let index = toolchains
            .iter()
            .enumerate()
            .find(|(_, toolchain)| toolchain == &&component.toolchain)
            .map(|(idx, _)| idx)
            .unwrap_or_else(|| {
                toolchains.push(component.toolchain.clone());
                result.push(vec![]);
                toolchains.len() - 1
            });
        result
            .get_mut(index)
            .expect(
                "either the index already existed or we just pushed the element with that index",
            )
            .push(component);
    }

    result
}

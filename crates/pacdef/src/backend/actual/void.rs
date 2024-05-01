use std::process::Command;

use anyhow::Result;
use regex::Regex;

use crate::backend::root::build_base_command_with_privileges;
use crate::cmd::run_external_command;
use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Void {}
impl Void {
    pub fn new() -> Self {
        Self {}
    }

    fn no_confirm_flag(no_confirm: bool) -> &str {
        if no_confirm {
            "-y"
        } else {
""
        }
    }
}
impl Default for Void {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MakeDependency;

impl Backend for Void {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    type Modification = MakeDependency;

    fn query_installed_packages(
        config: &Config,
    ) -> Result<std::collections::BTreeMap<Self::PackageId, Self::QueryInfo>> {
        // Removes the package status and description from output
        let re_str_1 = r"^ii |^uu |^hr |^\?\? | .*";
        // Removes the package version from output
        let re_str_2 = r"-[^-]*$";
        let re1 = Regex::new(re_str_1)?;
        let re2 = Regex::new(re_str_2)?;
        let mut cmd = Command::new("xbps-query");
        cmd.args(["-l"]);
        let output = String::from_utf8(cmd.output()?.stdout)?;

        let packages = output
            .lines()
            .map(|line| {
                let result = re1.replace_all(line, "");
                let result = re2.replace_all(&result, "");
                result.to_string().into()
            })
            .collect();
        Ok(packages)
    }

    fn install_packages(
        packages: &std::collections::BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        BackendInfo {
            binary: "xbps-install".to_string(),
            section: "void",
            switches_info: &[],
            switches_install: &["-S"],
            switches_no_confirm: &["-y"],
            switches_remove: &["-R"],
            switches_make_dependency: Some(&["-m", "auto"]),
        };

        let cmd = format!("xbps-install -S {} {}", Self::no_confirm_flag(no_confirm))

        let mut cmd = build_base_command_with_privileges("xbps-install");
        cmd.args(backend_info.switches_install);

        if no_confirm {
            cmd.args(backend_info.switches_no_confirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    fn remove_packages(
        packages: &std::collections::BTreeMap<Self::PackageId, Self::RemoveOptions>,
        no_confirm: bool,
    ) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = build_base_command_with_privileges("xbps-remove");
        cmd.args(backend_info.switches_remove);

        if no_confirm {
            cmd.args(backend_info.switches_no_confirm);
        }

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }

    fn modify_packages(
        packages: &std::collections::BTreeMap<Self::PackageId, Self::Modification>,
        config: &Config,
    ) -> Result<()> {
        let backend_info = self.backend_info();

        let mut cmd = build_base_command_with_privileges("xbps-pkgdb");
        cmd.args(
            backend_info
                .switches_make_dependency
                .expect("void should support make make dependency"),
        );

        for p in packages {
            cmd.arg(format!("{p}"));
        }

        run_external_command(cmd)
    }
}

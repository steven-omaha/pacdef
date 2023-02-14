use std::process::{ExitCode, Termination};

use anyhow::{Context, Result};

use pacdef_core as core;
use pacdef_core::{get_args, get_config_path, get_pacdef_group_dir, Config, Group, Pacdef};

fn main() -> ExitCode {
    handle_final_result(main_inner())
}

/// Skip printing the error chain when searching packages yields no results, otherwise report error
/// chain.
fn handle_final_result(result: Result<()>) -> ExitCode {
    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(ref e) => {
            let msg = e.root_cause().to_string();

            if msg == core::NO_PACKAGES_FOUND {
                ExitCode::FAILURE
            } else {
                result.report()
            }
        }
    }
}

fn main_inner() -> Result<()> {
    let args = get_args();

    let config_file = get_config_path().context("getting config file")?;
    let config = Config::load(&config_file).context("loading config file")?;

    let group_dir = get_pacdef_group_dir().context("resolving group dir")?;
    let groups = Group::load(&group_dir, config.warn_not_symlinks).context("loading groups")?;

    let pacdef = Pacdef::new(args, config, groups);
    pacdef.run_action_from_arg().context("running action")
}

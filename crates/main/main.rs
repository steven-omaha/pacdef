use std::process::{ExitCode, Termination};

use anyhow::{Context, Result};

use core::{args, Config, Group, Pacdef};

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
    let args = args::get();
    let config = Config::load().context("loading config file")?;
    let groups = Group::load(&config).context("loading groups").unwrap();
    let pacdef = Pacdef::new(args, config, groups);
    pacdef.run_action_from_arg().context("running action")
}

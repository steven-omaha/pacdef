use std::env::var;

use anyhow::{anyhow, Result};

pub(crate) fn get_editor() -> Result<String> {
    check_vars_in_order(&["EDITOR", "VISUAL"]).ok_or_else(|| anyhow!("could not find editor"))
}

fn check_vars_in_order(vars: &[&str]) -> Option<String> {
    vars.iter().flat_map(|v| var(v).ok()).next()
}

pub(crate) fn get_single_var(variable: &str) -> Option<String> {
    var(variable).ok()
}

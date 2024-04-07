use std::env::var;

use anyhow::{anyhow, Result};

pub fn get_editor() -> Result<String> {
    check_vars_in_order(&["EDITOR", "VISUAL"]).ok_or_else(|| anyhow!("could not find editor"))
}

fn check_vars_in_order(vars: &[&str]) -> Option<String> {
    vars.iter().find_map(|v| var(v).ok())
}

fn get_single_var(variable: &str) -> Option<String> {
    var(variable).ok()
}

pub fn should_print_debug_info() -> bool {
    match get_single_var("RUST_BACKTRACE") {
        Some(value) if ["s", "full"].contains(&value.as_str()) => true,
        Some(_) => false,
        None => false,
    }
}

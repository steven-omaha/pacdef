mod cli;
mod datastructure;
mod parsing;
#[cfg(test)]
mod tests;

pub use datastructure::*;

/// Get and parse the CLI arguments.
#[must_use]
pub fn get() -> Arguments {
    let args = cli::build_cli().get_matches();
    parsing::parse(args)
}

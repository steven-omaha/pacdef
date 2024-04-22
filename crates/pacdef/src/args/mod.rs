use self::datastructure::Arguments;

mod cli;
pub mod datastructure;
mod parsing;
#[cfg(test)]
mod tests;

/// Get and parse the CLI arguments.
#[must_use]
pub fn get() -> Arguments {
    let args = cli::build_cli().get_matches();
    parsing::parse(args)
}

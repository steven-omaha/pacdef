use std::error::Error as ErrorTrait;
use std::fmt::{Display, Write};
use std::path::PathBuf;

/// Error types for pacdef.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Package search yields no results.
    NoPackagesFound,
    /// Config file not found.
    ConfigFileNotFound,
    /// Group file not found.
    GroupFileNotFound(String),
    /// Group already exists.
    GroupAlreadyExists(PathBuf),
    /// Invalid group name ('.' or '..')
    InvalidGroupName(String),
    /// Multiple groups not found.
    MultipleGroupsNotFound(Vec<String>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPackagesFound => f.write_str("no packages matching query"),
            Self::ConfigFileNotFound => f.write_str("config file not found"),
            Self::GroupFileNotFound(name) => f.write_str(&format!("group file '{name}' not found")),
            Self::GroupAlreadyExists(path) => f.write_str(&format!(
                "group file '{}' already exists",
                path.to_string_lossy()
            )),
            Self::InvalidGroupName(name) => {
                f.write_str(&format!("group name '{name}' is not valid"))
            }
            Self::MultipleGroupsNotFound(vec) => {
                f.write_str("could not find the following groups:\n")?;
                let mut iter = vec.iter().peekable();
                while let Some(group) = iter.next() {
                    f.write_str(&format!("  {group}"))?;
                    if iter.peek().is_some() {
                        f.write_char('\n')?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl ErrorTrait for Error {}

use std::error::Error as ErrorTrait;
use std::fmt::Display;
use std::path::PathBuf;

/// Error types for pacdef.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
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
            Self::ConfigFileNotFound => write!(f, "config file not found"),
            Self::GroupFileNotFound(name) => write!(f, "group file '{name}' not found"),
            Self::GroupAlreadyExists(path) => {
                write!(f, "group file '{}' already exists", path.to_string_lossy())
            }
            Self::InvalidGroupName(name) => write!(f, "group name '{name}' is not valid"),
            Self::MultipleGroupsNotFound(vec) => {
                write!(
                    f,
                    "could not find the following groups: [{}]",
                    vec.join(", ")
                )
            }
        }
    }
}

impl ErrorTrait for Error {}

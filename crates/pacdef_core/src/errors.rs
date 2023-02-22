use std::error::Error as ErrorTrait;
use std::fmt::Display;

/// Error types for pacdef.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Package search yields no results.
    NoPackagesFound,
    /// No group files were provided as arguments.
    NoGroupFilesInArguments,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPackagesFound => f.write_str("no packages matching query"),
            Self::NoGroupFilesInArguments => f.write_str("no group files in arguments"),
        }
    }
}

impl ErrorTrait for Error {}

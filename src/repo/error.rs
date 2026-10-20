use std::path::PathBuf;
use thiserror::Error;

use crate::package::PackageError;

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("package `{name}` not found at `{}`", path.display())]
    NotFound { name: String, path: PathBuf },

    #[error("failed to read package `{name}` at `{}`: {source}", path.display())]
    Io {
        name: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse package `{name}` at `{}`: {source}", path.display())]
    Package {
        name: String,
        path: PathBuf,
        #[source]
        source: Box<PackageError>,
    },

    #[error("package name mismatch at `{}`: expected `{expected}`, found `{found}`", path.display())]
    NameMismatch {
        expected: String,
        found: String,
        path: PathBuf,
    },
}

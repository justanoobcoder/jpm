use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::consts;
use crate::package::{self, Package};

pub mod error;
pub use error::RepoError;

const PKGS_DIR_NAME: &str = "pkgs";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    pkgs_dir: PathBuf,
}

impl Default for Repo {
    fn default() -> Self {
        Self::from_env()
    }
}

impl Repo {
    pub fn from_env() -> Self {
        let base_dir = env::var_os(consts::MANIFEST_DIR_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(consts::env::JPM_MANIFEST_DIR));

        Self::new(base_dir.join(PKGS_DIR_NAME))
    }

    pub fn new(pkgs_dir: impl Into<PathBuf>) -> Self {
        Self {
            pkgs_dir: pkgs_dir.into(),
        }
    }

    pub fn pkgs_dir(&self) -> &Path {
        &self.pkgs_dir
    }

    pub fn package_path(&self, name: &str) -> PathBuf {
        let prefix = if name.len() >= 2 { &name[..2] } else { name };
        self.pkgs_dir
            .join(prefix)
            .join(name)
            .join(consts::MANIFEST_FILE)
    }

    pub fn get(&self, name: &str) -> Result<Package, RepoError> {
        let path = self.package_path(name);

        if !path.is_file() {
            return Err(RepoError::NotFound {
                name: name.to_string(),
                path,
            });
        }

        let content = fs::read_to_string(&path).map_err(|source| RepoError::Io {
            name: name.to_string(),
            path: path.clone(),
            source,
        })?;

        let pkg = package::parse(&content).map_err(|source| RepoError::Package {
            name: name.to_string(),
            path: path.clone(),
            source: Box::new(source),
        })?;

        if pkg.info.name != name {
            return Err(RepoError::NameMismatch {
                expected: name.to_string(),
                found: pkg.info.name,
                path,
            });
        }

        Ok(pkg)
    }
}

pub fn get_package(name: &str) -> Result<Package, RepoError> {
    Repo::from_env().get(name)
}

#[cfg(test)]
mod test;

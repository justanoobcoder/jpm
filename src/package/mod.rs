mod error;
mod model;
mod validate;

pub use error::{NameError, PackageError};
pub use model::{Build, Dependency, Hook, Info, Package};

pub fn parse(content: &str) -> Result<Package, PackageError> {
    let pkg: Package = toml::from_str(content)?;
    validate::validate(&pkg)?;
    Ok(pkg)
}

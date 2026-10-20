mod error;
mod model;
mod validator;

pub use error::{EnvError, FieldError, HashError, NameError, PackageError};
pub use model::{Build, BuildSystem, Dependency, Hook, Info, Package};

pub fn parse(content: &str) -> Result<Package, PackageError> {
    let pkg: Package = toml::from_str(content)?;
    validator::validate_package(&pkg)?;
    Ok(pkg)
}

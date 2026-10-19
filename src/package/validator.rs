use super::error::{FieldError, NameError, PackageError};
use super::model::Package;

pub const MAX_NAME_LEN: usize = 64;

fn validate_text(field: &'static str, value: &str) -> Result<(), PackageError> {
    if value.is_empty() {
        return Err(FieldError::Empty(field).into());
    }
    if value != value.trim() {
        return Err(FieldError::Whitespace(field).into());
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<(), PackageError> {
    validate_text("info.name", name)?;

    if let Some(c) = name
        .chars()
        .find(|&c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
    {
        return Err(NameError::InvalidChar(c).into());
    }
    if name.len() > MAX_NAME_LEN {
        return Err(NameError::TooLong.into());
    }
    if let Some(c) = name.chars().next() {
        if c == '_' || c == '-' {
            return Err(NameError::InvalidStartChar(c).into());
        }
    }
    Ok(())
}

pub fn validate(pkg: &Package) -> Result<(), PackageError> {
    let info = &pkg.info;

    validate_name(&info.name)?;
    validate_text("info.pkgver", &info.pkgver)?;
    validate_text("info.desc", &info.desc)?;
    validate_text("info.license", &info.license)?;
    validate_text("info.src", &info.src)?;
    validate_text("info.hash", &info.hash)?;

    if let Some(url) = &info.url {
        validate_text("info.url", url)?;
    }
    if let Some(dir) = &info.src_dir {
        validate_text("info.src_dir", dir)?;
    }

    Ok(())
}

use super::error::{FieldError, MAX_NAME_LEN, NameError, PackageError};
use super::model::Package;

pub fn validate_text(field: &'static str, value: &str) -> Result<(), FieldError> {
    if value.is_empty() {
        return Err(FieldError::Empty(field));
    }
    if value != value.trim() {
        return Err(FieldError::Whitespace(field));
    }
    Ok(())
}

pub fn validate_name(name: &str) -> Result<(), PackageError> {
    validate_text("info.name", name)?;

    if name.len() > MAX_NAME_LEN {
        return Err(NameError::TooLong.into());
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

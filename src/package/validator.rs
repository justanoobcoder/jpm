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
    if let Some(c @ ('_' | '-')) = name.chars().next() {
        return Err(NameError::InvalidStartChar(c).into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::model::{Build, BuildSystem, Info, Package};

    fn sample_package() -> Package {
        Package {
            info: Info {
                name: "btop".to_string(),
                pkgver: "1.4.7".to_string(),
                pkgrel: 1,
                desc: "A TUI monitor of resources".to_string(),
                license: "Apache-2.0".to_string(),
                url: Some("https://github.com/aristocratos/btop".to_string()),
                src: "https://github.com/aristocratos/btop/archive/v1.4.7/btop-1.4.7.tar.gz"
                    .to_string(),
                hash: "sha256:933de2e4d1b2211a638be463eb6e8616891bfba73aef5d38060bd8319baeefc6"
                    .to_string(),
                src_dir: Some("btop-1.4.7".to_string()),
            },
            dependency: None,
            hook: None,
            build: Build {
                system: BuildSystem::Make,
                configure: None,
                compile: None,
                install: Some("make install".to_string()),
                out_of_src: None,
            },
        }
    }

    #[test]
    fn test_validate_text_valid() {
        assert!(validate_text("field", "hello world").is_ok());
        assert!(validate_text("field", "123").is_ok());
        assert!(validate_text("field", "a").is_ok());
    }

    #[test]
    fn test_validate_text_empty() {
        let err = validate_text("test.field", "").unwrap_err();
        assert!(matches!(
            err,
            PackageError::Field(FieldError::Empty("test.field"))
        ));
    }

    #[test]
    fn test_validate_text_whitespace() {
        let err_leading = validate_text("test.field", " leading").unwrap_err();
        assert!(matches!(
            err_leading,
            PackageError::Field(FieldError::Whitespace("test.field"))
        ));

        let err_trailing = validate_text("test.field", "trailing ").unwrap_err();
        assert!(matches!(
            err_trailing,
            PackageError::Field(FieldError::Whitespace("test.field"))
        ));

        let err_tab = validate_text("test.field", "\tcontent").unwrap_err();
        assert!(matches!(
            err_tab,
            PackageError::Field(FieldError::Whitespace("test.field"))
        ));

        let err_newline = validate_text("test.field", "content\n").unwrap_err();
        assert!(matches!(
            err_newline,
            PackageError::Field(FieldError::Whitespace("test.field"))
        ));
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(validate_name("a").is_ok());
        assert!(validate_name("Z").is_ok());
        assert!(validate_name("9").is_ok());
        assert!(validate_name("btop").is_ok());
        assert!(validate_name("gcc-core").is_ok());
        assert!(validate_name("linux_kernel").is_ok());
        assert!(validate_name("lib32-gcc_9").is_ok());

        let max_len_name = "a".repeat(MAX_NAME_LEN);
        assert!(validate_name(&max_len_name).is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        let err = validate_name("").unwrap_err();
        assert!(matches!(
            err,
            PackageError::Field(FieldError::Empty("info.name"))
        ));
    }

    #[test]
    fn test_validate_name_whitespace() {
        let err = validate_name(" btop").unwrap_err();
        assert!(matches!(
            err,
            PackageError::Field(FieldError::Whitespace("info.name"))
        ));

        let err2 = validate_name("btop ").unwrap_err();
        assert!(matches!(
            err2,
            PackageError::Field(FieldError::Whitespace("info.name"))
        ));
    }

    #[test]
    fn test_validate_name_too_long() {
        let too_long = "a".repeat(MAX_NAME_LEN + 1);
        let err = validate_name(&too_long).unwrap_err();
        assert!(matches!(err, PackageError::InvalidName(NameError::TooLong)));
    }

    #[test]
    fn test_validate_name_invalid_chars() {
        let err_dot = validate_name("btop.tar").unwrap_err();
        assert!(matches!(
            err_dot,
            PackageError::InvalidName(NameError::InvalidChar('.'))
        ));

        let err_at = validate_name("btop@1").unwrap_err();
        assert!(matches!(
            err_at,
            PackageError::InvalidName(NameError::InvalidChar('@'))
        ));

        let err_slash = validate_name("btop/core").unwrap_err();
        assert!(matches!(
            err_slash,
            PackageError::InvalidName(NameError::InvalidChar('/'))
        ));

        let err_space = validate_name("btop core").unwrap_err();
        assert!(matches!(
            err_space,
            PackageError::InvalidName(NameError::InvalidChar(' '))
        ));
    }

    #[test]
    fn test_validate_name_invalid_start_char() {
        let err_dash = validate_name("-btop").unwrap_err();
        assert!(matches!(
            err_dash,
            PackageError::InvalidName(NameError::InvalidStartChar('-'))
        ));

        let err_under = validate_name("_btop").unwrap_err();
        assert!(matches!(
            err_under,
            PackageError::InvalidName(NameError::InvalidStartChar('_'))
        ));
    }

    #[test]
    fn test_validate_package_valid() {
        let pkg = sample_package();
        assert!(validate(&pkg).is_ok());
    }

    #[test]
    fn test_validate_package_without_optional_fields() {
        let mut pkg = sample_package();
        pkg.info.url = None;
        pkg.info.src_dir = None;
        assert!(validate(&pkg).is_ok());
    }

    #[test]
    fn test_validate_package_empty_fields() {
        let mut pkg = sample_package();
        pkg.info.pkgver = String::new();
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.pkgver"))
        ));

        let mut pkg = sample_package();
        pkg.info.desc = String::new();
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.desc"))
        ));

        let mut pkg = sample_package();
        pkg.info.license = String::new();
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.license"))
        ));

        let mut pkg = sample_package();
        pkg.info.src = String::new();
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.src"))
        ));

        let mut pkg = sample_package();
        pkg.info.hash = String::new();
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.hash"))
        ));
    }

    #[test]
    fn test_validate_package_whitespace_fields() {
        let mut pkg = sample_package();
        pkg.info.pkgver = " 1.4.7 ".to_string();
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.pkgver"))
        ));

        let mut pkg = sample_package();
        pkg.info.desc = "desc ".to_string();
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.desc"))
        ));

        let mut pkg = sample_package();
        pkg.info.license = " MIT".to_string();
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.license"))
        ));

        let mut pkg = sample_package();
        pkg.info.src = " https://example.com/src.tar.gz".to_string();
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.src"))
        ));

        let mut pkg = sample_package();
        pkg.info.hash = "sha256:123 ".to_string();
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.hash"))
        ));
    }

    #[test]
    fn test_validate_package_optional_url_invalid() {
        let mut pkg = sample_package();
        pkg.info.url = Some(String::new());
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.url"))
        ));

        pkg.info.url = Some(" https://example.com".to_string());
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.url"))
        ));
    }

    #[test]
    fn test_validate_package_optional_src_dir_invalid() {
        let mut pkg = sample_package();
        pkg.info.src_dir = Some(String::new());
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.src_dir"))
        ));

        pkg.info.src_dir = Some("btop-1.4.7 ".to_string());
        assert!(matches!(
            validate(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.src_dir"))
        ));
    }
}

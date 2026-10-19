use super::error::{FieldError, NameError, PackageError};
use super::model::Package;

pub const MAX_NAME_LEN: usize = 64;
const ALGOS: &[(&str, usize)] = &[("md5", 32), ("sha1", 40), ("sha256", 64)];

fn validate_text(field: &'static str, value: &str) -> Result<(), PackageError> {
    if value.is_empty() {
        return Err(FieldError::Empty(field).into());
    }
    if value != value.trim() {
        return Err(FieldError::Whitespace(field).into());
    }
    Ok(())
}

fn validate_name(field: &'static str, name: &str) -> Result<(), PackageError> {
    validate_text(field, name)?;

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

fn validate_hash(hash: &str) -> Result<(), PackageError> {
    validate_text("info.hash", hash)?;

    if hash.eq_ignore_ascii_case("skip") {
        return Ok(());
    }

    let (algo, digest) = hash
        .split_once(':')
        .ok_or_else(|| PackageError::InvalidHash(format!("expected `algo:hash`, got `{hash}`")))?;

    let &(_, expected_len) = ALGOS
        .iter()
        .find(|(name, _)| *name == algo)
        .ok_or_else(|| {
            let names: Vec<_> = ALGOS.iter().map(|(n, _)| *n).collect();
            PackageError::InvalidHash(format!(
                "unsupported algorithm `{algo}` (supported: {})",
                names.join(", ")
            ))
        })?;

    if !digest.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(PackageError::InvalidHash(
            "hash must contain hex characters only".into(),
        ));
    }

    if digest.len() != expected_len {
        return Err(PackageError::InvalidHash(format!(
            "expected {algo} hash length is {expected_len}, got {}",
            digest.len()
        )));
    }

    Ok(())
}

fn validate_dependency(field: &'static str, deps: &Vec<String>) -> Result<(), PackageError> {
    if deps.is_empty() {
        return Ok(());
    }
    for dep in deps {
        validate_name(field, dep)?;
    }

    Ok(())
}

pub fn validate_package(pkg: &Package) -> Result<(), PackageError> {
    let info = &pkg.info;

    validate_name("info.name", &info.name)?;
    validate_text("info.pkgver", &info.pkgver)?;
    validate_text("info.desc", &info.desc)?;
    validate_text("info.license", &info.license)?;
    validate_text("info.src", &info.src)?;
    validate_hash(&info.hash)?;

    if let Some(url) = &info.url {
        validate_text("info.url", url)?;
    }
    if let Some(dir) = &info.src_dir {
        validate_text("info.src_dir", dir)?;
    }

    if let Some(dep) = &pkg.dependency {
        validate_dependency("dependency.build", &dep.build)?;
        validate_dependency("dependency.runtime", &dep.runtime)?;
        validate_dependency("dependency.optional", &dep.optional)?;
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
        assert!(validate_name("info.name", "a").is_ok());
        assert!(validate_name("info.name", "Z").is_ok());
        assert!(validate_name("info.name", "9").is_ok());
        assert!(validate_name("info.name", "btop").is_ok());
        assert!(validate_name("info.name", "gcc-core").is_ok());
        assert!(validate_name("info.name", "linux_kernel").is_ok());
        assert!(validate_name("info.name", "lib32-gcc_9").is_ok());

        let max_len_name = "a".repeat(MAX_NAME_LEN);
        assert!(validate_name("info.name", &max_len_name).is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        let err = validate_name("info.name", "").unwrap_err();
        assert!(matches!(
            err,
            PackageError::Field(FieldError::Empty("info.name"))
        ));
    }

    #[test]
    fn test_validate_name_whitespace() {
        let err = validate_name("info.name", " btop").unwrap_err();
        assert!(matches!(
            err,
            PackageError::Field(FieldError::Whitespace("info.name"))
        ));

        let err2 = validate_name("info.name", "btop ").unwrap_err();
        assert!(matches!(
            err2,
            PackageError::Field(FieldError::Whitespace("info.name"))
        ));
    }

    #[test]
    fn test_validate_name_too_long() {
        let too_long = "a".repeat(MAX_NAME_LEN + 1);
        let err = validate_name("info.name", &too_long).unwrap_err();
        assert!(matches!(err, PackageError::InvalidName(NameError::TooLong)));
    }

    #[test]
    fn test_validate_name_invalid_chars() {
        let err_dot = validate_name("info.name", "btop.tar").unwrap_err();
        assert!(matches!(
            err_dot,
            PackageError::InvalidName(NameError::InvalidChar('.'))
        ));

        let err_at = validate_name("info.name", "btop@1").unwrap_err();
        assert!(matches!(
            err_at,
            PackageError::InvalidName(NameError::InvalidChar('@'))
        ));

        let err_slash = validate_name("info.name", "btop/core").unwrap_err();
        assert!(matches!(
            err_slash,
            PackageError::InvalidName(NameError::InvalidChar('/'))
        ));

        let err_space = validate_name("info.name", "btop core").unwrap_err();
        assert!(matches!(
            err_space,
            PackageError::InvalidName(NameError::InvalidChar(' '))
        ));
    }

    #[test]
    fn test_validate_name_invalid_start_char() {
        let err_dash = validate_name("info.name", "-btop").unwrap_err();
        assert!(matches!(
            err_dash,
            PackageError::InvalidName(NameError::InvalidStartChar('-'))
        ));

        let err_under = validate_name("info.name", "_btop").unwrap_err();
        assert!(matches!(
            err_under,
            PackageError::InvalidName(NameError::InvalidStartChar('_'))
        ));
    }

    #[test]
    fn test_validate_package_valid() {
        let pkg = sample_package();
        assert!(validate_package(&pkg).is_ok());
    }

    #[test]
    fn test_validate_package_without_optional_fields() {
        let mut pkg = sample_package();
        pkg.info.url = None;
        pkg.info.src_dir = None;
        assert!(validate_package(&pkg).is_ok());
    }

    #[test]
    fn test_validate_package_empty_fields() {
        let mut pkg = sample_package();
        pkg.info.pkgver = String::new();
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.pkgver"))
        ));

        let mut pkg = sample_package();
        pkg.info.desc = String::new();
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.desc"))
        ));

        let mut pkg = sample_package();
        pkg.info.license = String::new();
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.license"))
        ));

        let mut pkg = sample_package();
        pkg.info.src = String::new();
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.src"))
        ));

        let mut pkg = sample_package();
        pkg.info.hash = String::new();
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.hash"))
        ));
    }

    #[test]
    fn test_validate_package_whitespace_fields() {
        let mut pkg = sample_package();
        pkg.info.pkgver = " 1.4.7 ".to_string();
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.pkgver"))
        ));

        let mut pkg = sample_package();
        pkg.info.desc = "desc ".to_string();
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.desc"))
        ));

        let mut pkg = sample_package();
        pkg.info.license = " MIT".to_string();
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.license"))
        ));

        let mut pkg = sample_package();
        pkg.info.src = " https://example.com/src.tar.gz".to_string();
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.src"))
        ));

        let mut pkg = sample_package();
        pkg.info.hash = "sha256:123 ".to_string();
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.hash"))
        ));
    }

    #[test]
    fn test_validate_package_optional_url_invalid() {
        let mut pkg = sample_package();
        pkg.info.url = Some(String::new());
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.url"))
        ));

        pkg.info.url = Some(" https://example.com".to_string());
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.url"))
        ));
    }

    #[test]
    fn test_validate_package_optional_src_dir_invalid() {
        let mut pkg = sample_package();
        pkg.info.src_dir = Some(String::new());
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Empty("info.src_dir"))
        ));

        pkg.info.src_dir = Some("btop-1.4.7 ".to_string());
        assert!(matches!(
            validate_package(&pkg).unwrap_err(),
            PackageError::Field(FieldError::Whitespace("info.src_dir"))
        ));
    }

    #[test]
    fn test_validate_hash_valid() {
        assert!(validate_hash("md5:d41d8cd98f00b204e9800998ecf8427e").is_ok());
        assert!(validate_hash("sha1:da39a3ee5e6b4b0d3255bfef95601890afd80709").is_ok());
        assert!(
            validate_hash(
                "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            )
            .is_ok()
        );
        assert!(
            validate_hash(
                "sha256:E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
            )
            .is_ok()
        );
    }

    #[test]
    fn test_validate_hash_skip() {
        assert!(validate_hash("skip").is_ok());
        assert!(validate_hash("SKIP").is_ok());
        assert!(validate_hash("Skip").is_ok());
        assert!(validate_hash("sKiP").is_ok());
    }

    #[test]
    fn test_validate_hash_empty_and_whitespace() {
        let err_empty = validate_hash("").unwrap_err();
        assert!(matches!(
            err_empty,
            PackageError::Field(FieldError::Empty("info.hash"))
        ));

        let err_leading = validate_hash(
            " sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        )
        .unwrap_err();
        assert!(matches!(
            err_leading,
            PackageError::Field(FieldError::Whitespace("info.hash"))
        ));

        let err_trailing = validate_hash(
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 ",
        )
        .unwrap_err();
        assert!(matches!(
            err_trailing,
            PackageError::Field(FieldError::Whitespace("info.hash"))
        ));
    }

    #[test]
    fn test_validate_hash_missing_colon() {
        let err = validate_hash("sha256").unwrap_err();
        assert!(matches!(
            err,
            PackageError::InvalidHash(msg) if msg == "expected `algo:hash`, got `sha256`"
        ));

        let err_raw =
            validate_hash("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
                .unwrap_err();
        assert!(matches!(
            err_raw,
            PackageError::InvalidHash(msg) if msg.contains("expected `algo:hash`")
        ));
    }

    #[test]
    fn test_validate_hash_unsupported_algorithm() {
        let err_sha512 = validate_hash("sha512:1234").unwrap_err();
        assert!(matches!(
            err_sha512,
            PackageError::InvalidHash(msg) if msg == "unsupported algorithm `sha512` (supported: md5, sha1, sha256)"
        ));

        let err_blake3 = validate_hash("blake3:1234").unwrap_err();
        assert!(matches!(
            err_blake3,
            PackageError::InvalidHash(msg) if msg == "unsupported algorithm `blake3` (supported: md5, sha1, sha256)"
        ));
    }

    #[test]
    fn test_validate_hash_non_hex_digest() {
        let err_g = validate_hash("md5:d41d8cd98f00b204e9800998ecf8427g").unwrap_err();
        assert!(matches!(
            err_g,
            PackageError::InvalidHash(msg) if msg == "hash must contain hex characters only"
        ));

        let err_sym = validate_hash("sha1:da39a3ee5e6b4b0d3255bfef95601890afd8070!").unwrap_err();
        assert!(matches!(
            err_sym,
            PackageError::InvalidHash(msg) if msg == "hash must contain hex characters only"
        ));
    }

    #[test]
    fn test_validate_hash_length_mismatch() {
        let err_md5_short = validate_hash("md5:d41d8cd98f00b204e9800998ecf8427").unwrap_err();
        assert!(matches!(
            err_md5_short,
            PackageError::InvalidHash(msg) if msg == "expected md5 hash length is 32, got 31"
        ));

        let err_md5_long = validate_hash("md5:d41d8cd98f00b204e9800998ecf8427ea").unwrap_err();
        assert!(matches!(
            err_md5_long,
            PackageError::InvalidHash(msg) if msg == "expected md5 hash length is 32, got 33"
        ));

        let err_sha1_short =
            validate_hash("sha1:da39a3ee5e6b4b0d3255bfef95601890afd8070").unwrap_err();
        assert!(matches!(
            err_sha1_short,
            PackageError::InvalidHash(msg) if msg == "expected sha1 hash length is 40, got 39"
        ));

        let err_sha256_short =
            validate_hash("sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85")
                .unwrap_err();
        assert!(matches!(
            err_sha256_short,
            PackageError::InvalidHash(msg) if msg == "expected sha256 hash length is 64, got 63"
        ));

        let err_sha256_long = validate_hash(
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855a",
        )
        .unwrap_err();
        assert!(matches!(
            err_sha256_long,
            PackageError::InvalidHash(msg) if msg == "expected sha256 hash length is 64, got 65"
        ));
    }
}

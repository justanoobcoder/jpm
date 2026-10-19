use super::*;
use crate::package::model::{Build, BuildSystem, Dependency, Info, Package};

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
        validate_hash("sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
            .is_ok()
    );
    assert!(
        validate_hash("sha256:E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855")
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

    let err_leading =
        validate_hash(" sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
            .unwrap_err();
    assert!(matches!(
        err_leading,
        PackageError::Field(FieldError::Whitespace("info.hash"))
    ));

    let err_trailing =
        validate_hash("sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 ")
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

    let err_raw = validate_hash("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
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

    let err_sha1_short = validate_hash("sha1:da39a3ee5e6b4b0d3255bfef95601890afd8070").unwrap_err();
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

    let err_sha256_long =
        validate_hash("sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855a")
            .unwrap_err();
    assert!(matches!(
        err_sha256_long,
        PackageError::InvalidHash(msg) if msg == "expected sha256 hash length is 64, got 65"
    ));
}

#[test]
fn test_validate_dependency_empty() {
    let deps = vec![];
    assert!(validate_dependency("dependency.build", &deps).is_ok());
}

#[test]
fn test_validate_dependency_valid() {
    let deps = vec![
        "cmake".to_string(),
        "gcc-core".to_string(),
        "lib32_zlib".to_string(),
        "a".repeat(MAX_NAME_LEN),
    ];
    assert!(validate_dependency("dependency.build", &deps).is_ok());
}

#[test]
fn test_validate_dependency_empty_item() {
    let deps = vec!["cmake".to_string(), "".to_string()];
    let err = validate_dependency("dependency.build", &deps).unwrap_err();
    assert!(matches!(
        err,
        PackageError::Field(FieldError::Empty("dependency.build"))
    ));
}

#[test]
fn test_validate_dependency_whitespace_item() {
    let deps = vec![" gcc".to_string()];
    let err = validate_dependency("dependency.runtime", &deps).unwrap_err();
    assert!(matches!(
        err,
        PackageError::Field(FieldError::Whitespace("dependency.runtime"))
    ));

    let deps_trailing = vec!["gcc ".to_string()];
    let err_trailing = validate_dependency("dependency.optional", &deps_trailing).unwrap_err();
    assert!(matches!(
        err_trailing,
        PackageError::Field(FieldError::Whitespace("dependency.optional"))
    ));
}

#[test]
fn test_validate_dependency_invalid_chars() {
    let deps_dot = vec!["libfoo.so".to_string()];
    let err_dot = validate_dependency("dependency.runtime", &deps_dot).unwrap_err();
    assert!(matches!(
        err_dot,
        PackageError::InvalidName(NameError::InvalidChar('.'))
    ));

    let deps_at = vec!["pkg@1".to_string()];
    let err_at = validate_dependency("dependency.build", &deps_at).unwrap_err();
    assert!(matches!(
        err_at,
        PackageError::InvalidName(NameError::InvalidChar('@'))
    ));

    let deps_space = vec!["pkg name".to_string()];
    let err_space = validate_dependency("dependency.optional", &deps_space).unwrap_err();
    assert!(matches!(
        err_space,
        PackageError::InvalidName(NameError::InvalidChar(' '))
    ));
}

#[test]
fn test_validate_dependency_invalid_start_char() {
    let deps_dash = vec!["-pkg".to_string()];
    let err_dash = validate_dependency("dependency.build", &deps_dash).unwrap_err();
    assert!(matches!(
        err_dash,
        PackageError::InvalidName(NameError::InvalidStartChar('-'))
    ));

    let deps_under = vec!["_pkg".to_string()];
    let err_under = validate_dependency("dependency.runtime", &deps_under).unwrap_err();
    assert!(matches!(
        err_under,
        PackageError::InvalidName(NameError::InvalidStartChar('_'))
    ));
}

#[test]
fn test_validate_dependency_too_long() {
    let deps_long = vec!["a".repeat(MAX_NAME_LEN + 1)];
    let err = validate_dependency("dependency.build", &deps_long).unwrap_err();
    assert!(matches!(err, PackageError::InvalidName(NameError::TooLong)));
}

#[test]
fn test_validate_package_dependencies() {
    let mut pkg = sample_package();
    pkg.dependency = Some(Dependency {
        build: vec!["cmake".to_string()],
        runtime: vec!["glibc".to_string()],
        optional: vec!["wayland".to_string()],
    });
    assert!(validate_package(&pkg).is_ok());

    let mut pkg_build_err = sample_package();
    pkg_build_err.dependency = Some(Dependency {
        build: vec!["".to_string()],
        runtime: vec![],
        optional: vec![],
    });
    assert!(matches!(
        validate_package(&pkg_build_err).unwrap_err(),
        PackageError::Field(FieldError::Empty("dependency.build"))
    ));

    let mut pkg_runtime_err = sample_package();
    pkg_runtime_err.dependency = Some(Dependency {
        build: vec![],
        runtime: vec![" glibc".to_string()],
        optional: vec![],
    });
    assert!(matches!(
        validate_package(&pkg_runtime_err).unwrap_err(),
        PackageError::Field(FieldError::Whitespace("dependency.runtime"))
    ));

    let mut pkg_optional_err = sample_package();
    pkg_optional_err.dependency = Some(Dependency {
        build: vec![],
        runtime: vec![],
        optional: vec!["-wayland".to_string()],
    });
    assert!(matches!(
        validate_package(&pkg_optional_err).unwrap_err(),
        PackageError::InvalidName(NameError::InvalidStartChar('-'))
    ));
}

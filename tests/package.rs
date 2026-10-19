use jpm::package::{BuildSystem, FieldError, NameError, PackageError, parse};

const VALID_MINIMAL_MANIFEST: &str = r#"
[info]
name = "btop"
pkgver = "1.4.7"
pkgrel = 1
desc = "A TUI monitor of resources"
license = "Apache-2.0"
src = "https://example.com/btop-1.4.7.tar.gz"
hash = "sha256:1234567890abcdef"

[build]
system = "make"
"#;

const VALID_FULL_MANIFEST: &str = r#"
[info]
name = "btop"
pkgver = "1.4.7"
pkgrel = 1
desc = "A TUI monitor of resources"
license = "Apache-2.0"
url = "https://github.com/aristocratos/btop"
src = "https://github.com/aristocratos/btop/archive/v1.4.7/btop-1.4.7.tar.gz"
hash = "sha256:933de2e4d1b2211a638be463eb6e8616891bfba73aef5d38060bd8319baeefc6"
src_dir = "btop-1.4.7"

[dependency]
build = ["cmake", "gcc"]
runtime = ["glibc"]
optional = ["rocm"]

[build]
system = "make"
configure = "./configure --prefix=/usr"
compile = "make -j$(nproc)"
install = "make DESTDIR=$DESTDIR PREFIX=/usr install"
out_of_src = false

[hook]
patches = "patch -p1 < fix.patch"
pre_configure = "echo pre_configure"
pre_install = "echo pre_install"
post_install = "echo post_install"
pre_remove = "echo pre_remove"
post_remove = "echo post_remove"
"#;

#[test]
fn test_parse_minimal_manifest() {
    let pkg = parse(VALID_MINIMAL_MANIFEST).expect("minimal manifest should parse successfully");

    assert_eq!(pkg.info.name, "btop");
    assert_eq!(pkg.info.pkgver, "1.4.7");
    assert_eq!(pkg.info.pkgrel, 1);
    assert_eq!(pkg.info.desc, "A TUI monitor of resources");
    assert_eq!(pkg.info.license, "Apache-2.0");
    assert_eq!(pkg.info.src, "https://example.com/btop-1.4.7.tar.gz");
    assert_eq!(pkg.info.hash, "sha256:1234567890abcdef");
    assert_eq!(pkg.info.url, None);
    assert_eq!(pkg.info.src_dir, None);

    assert!(pkg.dependency.is_none());
    assert!(pkg.hook.is_none());

    assert_eq!(pkg.build.system, BuildSystem::Make);
    assert_eq!(pkg.build.configure, None);
    assert_eq!(pkg.build.compile, None);
    assert_eq!(pkg.build.install, None);
    assert_eq!(pkg.build.out_of_src, None);
}

#[test]
fn test_parse_full_manifest() {
    let pkg = parse(VALID_FULL_MANIFEST).expect("full manifest should parse successfully");

    assert_eq!(pkg.info.name, "btop");
    assert_eq!(pkg.info.pkgver, "1.4.7");
    assert_eq!(pkg.info.pkgrel, 1);
    assert_eq!(pkg.info.desc, "A TUI monitor of resources");
    assert_eq!(pkg.info.license, "Apache-2.0");
    assert_eq!(
        pkg.info.url.as_deref(),
        Some("https://github.com/aristocratos/btop")
    );
    assert_eq!(
        pkg.info.src,
        "https://github.com/aristocratos/btop/archive/v1.4.7/btop-1.4.7.tar.gz"
    );
    assert_eq!(
        pkg.info.hash,
        "sha256:933de2e4d1b2211a638be463eb6e8616891bfba73aef5d38060bd8319baeefc6"
    );
    assert_eq!(pkg.info.src_dir.as_deref(), Some("btop-1.4.7"));

    let dep = pkg.dependency.expect("dependency should be present");
    assert_eq!(dep.build, vec!["cmake", "gcc"]);
    assert_eq!(dep.runtime, vec!["glibc"]);
    assert_eq!(dep.optional, vec!["rocm"]);

    assert_eq!(pkg.build.system, BuildSystem::Make);
    assert_eq!(
        pkg.build.configure.as_deref(),
        Some("./configure --prefix=/usr")
    );
    assert_eq!(pkg.build.compile.as_deref(), Some("make -j$(nproc)"));
    assert_eq!(
        pkg.build.install.as_deref(),
        Some("make DESTDIR=$DESTDIR PREFIX=/usr install")
    );
    assert_eq!(pkg.build.out_of_src, Some(false));

    let hook = pkg.hook.expect("hook should be present");
    assert_eq!(hook.patches.as_deref(), Some("patch -p1 < fix.patch"));
    assert_eq!(hook.pre_configure.as_deref(), Some("echo pre_configure"));
    assert_eq!(hook.pre_install.as_deref(), Some("echo pre_install"));
    assert_eq!(hook.post_install.as_deref(), Some("echo post_install"));
    assert_eq!(hook.pre_remove.as_deref(), Some("echo pre_remove"));
    assert_eq!(hook.post_remove.as_deref(), Some("echo post_remove"));
}

#[test]
fn test_parse_all_build_systems() {
    let systems = [
        ("autotools", BuildSystem::Autotools),
        ("cmake", BuildSystem::Cmake),
        ("meson", BuildSystem::Meson),
        ("make", BuildSystem::Make),
        ("custom", BuildSystem::Custom),
    ];

    for (name, expected_system) in systems {
        let toml = format!(
            r#"
[info]
name = "test-pkg"
pkgver = "1.0"
pkgrel = 1
desc = "test"
license = "MIT"
src = "https://example.com/src.tar.gz"
hash = "sha256:123"

[build]
system = "{name}"
"#
        );
        let pkg =
            parse(&toml).unwrap_or_else(|e| panic!("failed to parse build system {name}: {e}"));
        assert_eq!(pkg.build.system, expected_system);
    }
}

#[test]
fn test_parse_invalid_build_system() {
    let toml = r#"
[info]
name = "test-pkg"
pkgver = "1.0"
pkgrel = 1
desc = "test"
license = "MIT"
src = "https://example.com/src.tar.gz"
hash = "sha256:123"

[build]
system = "unsupported_system"
"#;
    let err = parse(toml).unwrap_err();
    assert!(matches!(err, PackageError::Parse(_)));
}

#[test]
fn test_parse_empty_dependency_table() {
    let toml = r#"
[info]
name = "test-pkg"
pkgver = "1.0"
pkgrel = 1
desc = "test"
license = "MIT"
src = "https://example.com/src.tar.gz"
hash = "sha256:123"

[dependency]

[build]
system = "make"
"#;
    let pkg = parse(toml).expect("empty dependency table should parse");
    let dep = pkg.dependency.expect("dependency should be Some");
    assert!(dep.build.is_empty());
    assert!(dep.runtime.is_empty());
    assert!(dep.optional.is_empty());
}

#[test]
fn test_parse_invalid_toml_syntax() {
    let toml = r#"
[info
name = "unclosed bracket"
"#;
    let err = parse(toml).unwrap_err();
    assert!(matches!(err, PackageError::Parse(_)));
}

#[test]
fn test_parse_missing_info_section() {
    let toml = r#"
[build]
system = "make"
"#;
    let err = parse(toml).unwrap_err();
    assert!(matches!(err, PackageError::Parse(_)));
}

#[test]
fn test_parse_missing_build_section() {
    let toml = r#"
[info]
name = "test-pkg"
pkgver = "1.0"
pkgrel = 1
desc = "test"
license = "MIT"
src = "https://example.com/src.tar.gz"
hash = "sha256:123"
"#;
    let err = parse(toml).unwrap_err();
    assert!(matches!(err, PackageError::Parse(_)));
}

#[test]
fn test_parse_missing_required_info_fields() {
    let fields = ["name", "pkgver", "pkgrel", "desc", "license", "src", "hash"];

    for field in fields {
        let mut lines = vec![
            "[info]".to_string(),
            r#"name = "test-pkg""#.to_string(),
            r#"pkgver = "1.0""#.to_string(),
            "pkgrel = 1".to_string(),
            r#"desc = "test""#.to_string(),
            r#"license = "MIT""#.to_string(),
            r#"src = "https://example.com/src.tar.gz""#.to_string(),
            r#"hash = "sha256:123""#.to_string(),
            "[build]".to_string(),
            r#"system = "make""#.to_string(),
        ];
        lines.retain(|l| !l.starts_with(field));
        let toml = lines.join("\n");
        let err = parse(&toml).unwrap_err();
        assert!(
            matches!(err, PackageError::Parse(_)),
            "Expected parse error when missing {field}, got {err:?}"
        );
    }
}

#[test]
fn test_parse_type_mismatch() {
    let toml = r#"
[info]
name = "test-pkg"
pkgver = "1.0"
pkgrel = "one"
desc = "test"
license = "MIT"
src = "https://example.com/src.tar.gz"
hash = "sha256:123"

[build]
system = "make"
"#;
    let err = parse(toml).unwrap_err();
    assert!(matches!(err, PackageError::Parse(_)));
}

#[test]
fn test_parse_name_validation_errors() {
    // Starts with '-'
    let toml_dash = VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, r#"name = "-btop""#);
    assert_eq!(
        parse(&toml_dash).unwrap_err().to_string(),
        "Invalid package name: must not start with `-`"
    );

    // Starts with '_'
    let toml_under = VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, r#"name = "_btop""#);
    assert_eq!(
        parse(&toml_under).unwrap_err().to_string(),
        "Invalid package name: must not start with `_`"
    );

    // Invalid character
    let toml_invalid_char =
        VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, r#"name = "btop.pkg""#);
    assert_eq!(
        parse(&toml_invalid_char).unwrap_err().to_string(),
        "Invalid package name: invalid character `.` in name (allowed: a-z, A-Z, 0-9, `-`, `_`)"
    );

    // Empty name
    let toml_empty = VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, r#"name = """#);
    assert_eq!(
        parse(&toml_empty).unwrap_err().to_string(),
        "Invalid field value: `info.name` must not be empty"
    );

    // Whitespace name
    let toml_ws = VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, r#"name = " btop""#);
    assert_eq!(
        parse(&toml_ws).unwrap_err().to_string(),
        "Invalid field value: `info.name` must not have leading or trailing whitespace"
    );

    // Too long name (> 64 characters)
    let long_name = "a".repeat(65);
    let toml_too_long =
        VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, &format!(r#"name = "{long_name}""#));
    assert_eq!(
        parse(&toml_too_long).unwrap_err().to_string(),
        "Invalid package name: too long (max 64 characters)"
    );
}

#[test]
fn test_parse_field_validation_errors() {
    let test_cases = [
        ("pkgver", r#"pkgver = """#, FieldError::Empty("info.pkgver")),
        (
            "pkgver",
            r#"pkgver = " 1.0 ""#,
            FieldError::Whitespace("info.pkgver"),
        ),
        ("desc", r#"desc = """#, FieldError::Empty("info.desc")),
        (
            "desc",
            r#"desc = "desc ""#,
            FieldError::Whitespace("info.desc"),
        ),
        (
            "license",
            r#"license = """#,
            FieldError::Empty("info.license"),
        ),
        (
            "license",
            r#"license = " MIT""#,
            FieldError::Whitespace("info.license"),
        ),
        ("src", r#"src = """#, FieldError::Empty("info.src")),
        (
            "src",
            r#"src = " src ""#,
            FieldError::Whitespace("info.src"),
        ),
        ("hash", r#"hash = """#, FieldError::Empty("info.hash")),
        (
            "hash",
            r#"hash = "hash ""#,
            FieldError::Whitespace("info.hash"),
        ),
    ];

    for (field, replacement, expected_err) in test_cases {
        let orig_line = match field {
            "pkgver" => r#"pkgver = "1.4.7""#,
            "desc" => r#"desc = "A TUI monitor of resources""#,
            "license" => r#"license = "Apache-2.0""#,
            "src" => r#"src = "https://example.com/btop-1.4.7.tar.gz""#,
            "hash" => r#"hash = "sha256:1234567890abcdef""#,
            _ => unreachable!(),
        };

        let toml = VALID_MINIMAL_MANIFEST.replace(orig_line, replacement);
        let err = parse(&toml).unwrap_err();
        match err {
            PackageError::Field(e) => assert_eq!(e, expected_err, "Mismatch for field {field}"),
            other => panic!("Expected FieldError for {field}, got {other:?}"),
        }
    }
}

#[test]
fn test_parse_optional_fields_validation_errors() {
    // Empty url
    let toml_url_empty = VALID_MINIMAL_MANIFEST.replace("[build]", "url = \"\"\n[build]");
    let err = parse(&toml_url_empty).unwrap_err();
    assert!(matches!(
        err,
        PackageError::Field(FieldError::Empty("info.url"))
    ));

    // Whitespace url
    let toml_url_ws =
        VALID_MINIMAL_MANIFEST.replace("[build]", "url = \" https://example.com \"\n[build]");
    let err = parse(&toml_url_ws).unwrap_err();
    assert!(matches!(
        err,
        PackageError::Field(FieldError::Whitespace("info.url"))
    ));

    // Empty src_dir
    let toml_dir_empty = VALID_MINIMAL_MANIFEST.replace("[build]", "src_dir = \"\"\n[build]");
    let err = parse(&toml_dir_empty).unwrap_err();
    assert!(matches!(
        err,
        PackageError::Field(FieldError::Empty("info.src_dir"))
    ));

    // Whitespace src_dir
    let toml_dir_ws = VALID_MINIMAL_MANIFEST.replace("[build]", "src_dir = \" dir \"\n[build]");
    let err = parse(&toml_dir_ws).unwrap_err();
    assert!(matches!(
        err,
        PackageError::Field(FieldError::Whitespace("info.src_dir"))
    ));
}

#[test]
fn test_error_display() {
    let empty_err = FieldError::Empty("info.name");
    assert_eq!(empty_err.to_string(), "`info.name` must not be empty");

    let ws_err = FieldError::Whitespace("info.desc");
    assert_eq!(
        ws_err.to_string(),
        "`info.desc` must not have leading or trailing whitespace"
    );

    let too_long_err = NameError::TooLong;
    assert_eq!(too_long_err.to_string(), "too long (max 64 characters)");

    let invalid_char_err = NameError::InvalidChar('@');
    assert_eq!(
        invalid_char_err.to_string(),
        "invalid character `@` in name (allowed: a-z, A-Z, 0-9, `-`, `_`)"
    );

    let invalid_start_err = NameError::InvalidStartChar('-');
    assert_eq!(invalid_start_err.to_string(), "must not start with `-`");

    let pkg_field_err = PackageError::Field(empty_err);
    assert_eq!(
        pkg_field_err.to_string(),
        "Invalid field value: `info.name` must not be empty"
    );

    let pkg_name_err = PackageError::InvalidName(invalid_start_err);
    assert_eq!(
        pkg_name_err.to_string(),
        "Invalid package name: must not start with `-`"
    );
}

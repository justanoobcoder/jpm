use jpm::package::{BuildSystem, FieldError, NameError, PackageError, parse};

const VALID_MINIMAL_MANIFEST: &str = r#"
[info]
name = "btop"
pkgver = "1.4.7"
pkgrel = 1
desc = "A TUI monitor of resources"
license = "Apache-2.0"
src = "https://example.com/btop-1.4.7.tar.gz"
hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"

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
patches = ["fix.patch"]
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
    assert_eq!(
        pkg.info.hash,
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
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
    assert_eq!(hook.patches[0], "fix.patch");
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
hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"

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
hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"

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
hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"

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
hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
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
            r#"hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad""#
                .to_string(),
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
hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"

[build]
system = "make"
"#;
    let err = parse(toml).unwrap_err();
    assert!(matches!(err, PackageError::Parse(_)));
}

#[test]
fn test_parse_name_validation_errors() {
    let toml_dash = VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, r#"name = "-btop""#);
    assert_eq!(
        parse(&toml_dash).unwrap_err().to_string(),
        "Invalid package name: must not start with `-`"
    );

    let toml_under = VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, r#"name = "_btop""#);
    assert_eq!(
        parse(&toml_under).unwrap_err().to_string(),
        "Invalid package name: must not start with `_`"
    );

    let toml_invalid_char =
        VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, r#"name = "btop.pkg""#);
    assert_eq!(
        parse(&toml_invalid_char).unwrap_err().to_string(),
        "Invalid package name: invalid character `.` in name (allowed: a-z, A-Z, 0-9, `-`, `_`)"
    );

    let toml_empty = VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, r#"name = """#);
    assert_eq!(
        parse(&toml_empty).unwrap_err().to_string(),
        "Invalid field value: `info.name` must not be empty"
    );

    let toml_ws = VALID_MINIMAL_MANIFEST.replace(r#"name = "btop""#, r#"name = " btop""#);
    assert_eq!(
        parse(&toml_ws).unwrap_err().to_string(),
        "Invalid field value: `info.name` must not have leading or trailing whitespace"
    );

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
            "hash" => {
                r#"hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad""#
            }
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
    let toml_url_empty = VALID_MINIMAL_MANIFEST.replace("[build]", "url = \"\"\n[build]");
    let err = parse(&toml_url_empty).unwrap_err();
    assert!(matches!(
        err,
        PackageError::Field(FieldError::Empty("info.url"))
    ));

    let toml_url_ws =
        VALID_MINIMAL_MANIFEST.replace("[build]", "url = \" https://example.com \"\n[build]");
    let err = parse(&toml_url_ws).unwrap_err();
    assert!(matches!(
        err,
        PackageError::Field(FieldError::Whitespace("info.url"))
    ));

    let toml_dir_empty = VALID_MINIMAL_MANIFEST.replace("[build]", "src_dir = \"\"\n[build]");
    let err = parse(&toml_dir_empty).unwrap_err();
    assert!(matches!(
        err,
        PackageError::Field(FieldError::Empty("info.src_dir"))
    ));

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

    let pkg_hash_err = PackageError::InvalidHash("hash must contain hex characters only".into());
    assert_eq!(
        pkg_hash_err.to_string(),
        "Invalid hash: hash must contain hex characters only"
    );
}

#[test]
fn test_parse_hash_valid_formats() {
    let valid_hashes = [
        "md5:d41d8cd98f00b204e9800998ecf8427e",
        "sha1:da39a3ee5e6b4b0d3255bfef95601890afd80709",
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "sha256:E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        "skip",
        "SKIP",
        "Skip",
    ];

    for hash in valid_hashes {
        let toml = VALID_MINIMAL_MANIFEST.replace(
            r#"hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad""#,
            &format!(r#"hash = "{hash}""#),
        );
        let pkg = parse(&toml).unwrap_or_else(|e| panic!("failed to parse valid hash {hash}: {e}"));
        assert_eq!(pkg.info.hash, hash);
    }
}

#[test]
fn test_parse_hash_validation_errors() {
    let toml_no_colon = VALID_MINIMAL_MANIFEST.replace(
        r#"hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad""#,
        r#"hash = "sha256""#,
    );
    assert_eq!(
        parse(&toml_no_colon).unwrap_err().to_string(),
        "Invalid hash: expected `algo:hash`, got `sha256`"
    );

    let toml_unsupported = VALID_MINIMAL_MANIFEST.replace(
        r#"hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad""#,
        r#"hash = "sha512:1234""#,
    );
    assert_eq!(
        parse(&toml_unsupported).unwrap_err().to_string(),
        "Invalid hash: unsupported algorithm `sha512` (supported: md5, sha1, sha256)"
    );

    let toml_non_hex = VALID_MINIMAL_MANIFEST.replace(
        r#"hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad""#,
        r#"hash = "md5:d41d8cd98f00b204e9800998ecf8427g""#,
    );
    assert_eq!(
        parse(&toml_non_hex).unwrap_err().to_string(),
        "Invalid hash: hash must contain hex characters only"
    );

    let toml_bad_len = VALID_MINIMAL_MANIFEST.replace(
        r#"hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad""#,
        r#"hash = "sha256:1234""#,
    );
    assert_eq!(
        parse(&toml_bad_len).unwrap_err().to_string(),
        "Invalid hash: expected sha256 hash length is 64, got 4"
    );

    let toml_md5_short = VALID_MINIMAL_MANIFEST.replace(
        r#"hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad""#,
        r#"hash = "md5:1234""#,
    );
    assert_eq!(
        parse(&toml_md5_short).unwrap_err().to_string(),
        "Invalid hash: expected md5 hash length is 32, got 4"
    );

    let toml_sha1_short = VALID_MINIMAL_MANIFEST.replace(
        r#"hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad""#,
        r#"hash = "sha1:1234""#,
    );
    assert_eq!(
        parse(&toml_sha1_short).unwrap_err().to_string(),
        "Invalid hash: expected sha1 hash length is 40, got 4"
    );
}

#[test]
fn test_parse_dependency_valid_sections() {
    let toml = format!(
        r#"{VALID_MINIMAL_MANIFEST}
[dependency]
build = ["cmake", "gcc-core", "lib32_zlib"]
runtime = ["glibc", "systemd"]
optional = ["wayland", "x11"]
"#
    );
    let pkg = parse(&toml).expect("valid dependencies should parse successfully");
    let dep = pkg.dependency.expect("dependency should be Some");
    assert_eq!(dep.build, vec!["cmake", "gcc-core", "lib32_zlib"]);
    assert_eq!(dep.runtime, vec!["glibc", "systemd"]);
    assert_eq!(dep.optional, vec!["wayland", "x11"]);
}

#[test]
fn test_parse_dependency_validation_errors() {
    let toml_empty_build = format!(
        r#"{VALID_MINIMAL_MANIFEST}
[dependency]
build = ["cmake", ""]
"#
    );
    assert_eq!(
        parse(&toml_empty_build).unwrap_err().to_string(),
        "Invalid field value: `dependency.build` must not be empty"
    );

    let toml_ws_runtime = format!(
        r#"{VALID_MINIMAL_MANIFEST}
[dependency]
runtime = [" glibc"]
"#
    );
    assert_eq!(
        parse(&toml_ws_runtime).unwrap_err().to_string(),
        "Invalid field value: `dependency.runtime` must not have leading or trailing whitespace"
    );

    let toml_invalid_char = format!(
        r#"{VALID_MINIMAL_MANIFEST}
[dependency]
optional = ["wayland.lib"]
"#
    );
    assert_eq!(
        parse(&toml_invalid_char).unwrap_err().to_string(),
        "Invalid package name: invalid character `.` in name (allowed: a-z, A-Z, 0-9, `-`, `_`)"
    );

    let toml_start_dash = format!(
        r#"{VALID_MINIMAL_MANIFEST}
[dependency]
build = ["-cmake"]
"#
    );
    assert_eq!(
        parse(&toml_start_dash).unwrap_err().to_string(),
        "Invalid package name: must not start with `-`"
    );

    let toml_start_under = format!(
        r#"{VALID_MINIMAL_MANIFEST}
[dependency]
build = ["_cmake"]
"#
    );
    assert_eq!(
        parse(&toml_start_under).unwrap_err().to_string(),
        "Invalid package name: must not start with `_`"
    );

    let long_dep = "a".repeat(65);
    let toml_too_long = format!(
        r#"{VALID_MINIMAL_MANIFEST}
[dependency]
runtime = ["{long_dep}"]
"#
    );
    assert_eq!(
        parse(&toml_too_long).unwrap_err().to_string(),
        "Invalid package name: too long (max 64 characters)"
    );
}

#[test]
fn test_parse_hook_valid_sections() {
    let toml = format!(
        r#"{VALID_MINIMAL_MANIFEST}
[hook]
patches = ["0001-fix.patch", "security.patch"]
pre_configure = "sed -i 's/a/b/' Makefile"
pre_install = "mkdir -p $DESTDIR"
post_install = "ldconfig"
pre_remove = "echo removing"
post_remove = "echo removed"
"#
    );
    let pkg = parse(&toml).expect("valid hooks should parse successfully");
    let hook = pkg.hook.expect("hook should be Some");
    assert_eq!(hook.patches, vec!["0001-fix.patch", "security.patch"]);
    assert_eq!(
        hook.pre_configure.as_deref(),
        Some("sed -i 's/a/b/' Makefile")
    );
    assert_eq!(hook.pre_install.as_deref(), Some("mkdir -p $DESTDIR"));
    assert_eq!(hook.post_install.as_deref(), Some("ldconfig"));
    assert_eq!(hook.pre_remove.as_deref(), Some("echo removing"));
    assert_eq!(hook.post_remove.as_deref(), Some("echo removed"));
}

#[test]
fn test_parse_hook_validation_errors() {
    let toml_empty_patch = format!(
        r#"{VALID_MINIMAL_MANIFEST}
[hook]
patches = ["fix.patch", ""]
"#
    );
    assert_eq!(
        parse(&toml_empty_patch).unwrap_err().to_string(),
        "Invalid field value: `hook.patches` must not be empty"
    );

    let toml_ws_patch = format!(
        r#"{VALID_MINIMAL_MANIFEST}
[hook]
patches = [" fix.patch"]
"#
    );
    assert_eq!(
        parse(&toml_ws_patch).unwrap_err().to_string(),
        "Invalid field value: `hook.patches` must not have leading or trailing whitespace"
    );

    let empty_fields = [
        ("pre_configure", "hook.pre_configure"),
        ("pre_install", "hook.pre_install"),
        ("post_install", "hook.post_install"),
        ("pre_remove", "hook.pre_remove"),
        ("post_remove", "hook.post_remove"),
    ];

    for (field_name, expected_err_field) in empty_fields {
        let toml_empty_field = format!(
            r#"{VALID_MINIMAL_MANIFEST}
[hook]
{field_name} = ""
"#
        );
        assert_eq!(
            parse(&toml_empty_field).unwrap_err().to_string(),
            format!("Invalid field value: `{expected_err_field}` must not be empty")
        );
    }
}

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

fn temp_test_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = env::temp_dir().join(format!(
        "jpm_repo_test_{label}_{}_{nanos}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

const VALID_PACKAGE_TOML: &str = r#"
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

#[test]
fn test_package_path_structure() {
    let repo = Repo::new("/usr/share/jpm/pkgs");

    assert_eq!(
        repo.package_path("btop"),
        PathBuf::from("/usr/share/jpm/pkgs/bt/btop/package.toml")
    );
    assert_eq!(
        repo.package_path("cmake"),
        PathBuf::from("/usr/share/jpm/pkgs/cm/cmake/package.toml")
    );
    assert_eq!(
        repo.package_path("gcc-core"),
        PathBuf::from("/usr/share/jpm/pkgs/gc/gcc-core/package.toml")
    );
    assert_eq!(
        repo.package_path("a"),
        PathBuf::from("/usr/share/jpm/pkgs/a/a/package.toml")
    );
}

#[test]
fn test_repo_from_env_default() {
    let prev = env::var_os(consts::env::MANIFEST_DIR_ENV);
    unsafe {
        env::remove_var(consts::env::MANIFEST_DIR_ENV);
    }

    let repo = Repo::from_env();
    let expected = PathBuf::from(consts::env::JPM_MANIFEST_DIR).join(PKGS_DIR_NAME);
    assert_eq!(repo.pkgs_dir(), expected);

    if let Some(val) = prev {
        unsafe {
            env::set_var(consts::env::MANIFEST_DIR_ENV, val);
        }
    }
}

#[test]
fn test_repo_from_env_custom() {
    let prev = env::var_os(consts::env::MANIFEST_DIR_ENV);
    let custom_manifest_dir = "/tmp/custom_jpm";
    unsafe {
        env::set_var(consts::env::MANIFEST_DIR_ENV, custom_manifest_dir);
    }

    let repo = Repo::from_env();
    assert_eq!(
        repo.pkgs_dir(),
        PathBuf::from("/tmp/custom_jpm").join(PKGS_DIR_NAME)
    );

    if let Some(val) = prev {
        unsafe {
            env::set_var(consts::env::MANIFEST_DIR_ENV, val);
        }
    } else {
        unsafe {
            env::remove_var(consts::env::MANIFEST_DIR_ENV);
        }
    }
}

#[test]
fn test_repo_get_success() {
    let temp = temp_test_dir("get_success");
    let pkgs_dir = temp.join("pkgs");
    let repo = Repo::new(&pkgs_dir);

    let pkg_file = repo.package_path("btop");
    fs::create_dir_all(pkg_file.parent().unwrap()).unwrap();
    fs::write(&pkg_file, VALID_PACKAGE_TOML).unwrap();

    let pkg = repo
        .get("btop")
        .expect("package should be found and parsed");
    assert_eq!(pkg.info.name, "btop");
    assert_eq!(pkg.info.pkgver, "1.4.7");

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn test_repo_get_not_found() {
    let temp = temp_test_dir("not_found");
    let pkgs_dir = temp.join("pkgs");
    let repo = Repo::new(&pkgs_dir);

    let err = repo.get("nonexistent").unwrap_err();
    assert!(matches!(err, RepoError::NotFound { ref name, .. } if name == "nonexistent"));

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn test_repo_get_invalid_manifest() {
    let temp = temp_test_dir("invalid_manifest");
    let pkgs_dir = temp.join("pkgs");
    let repo = Repo::new(&pkgs_dir);

    let pkg_file = repo.package_path("btop");
    fs::create_dir_all(pkg_file.parent().unwrap()).unwrap();
    fs::write(&pkg_file, "invalid toml [[[").unwrap();

    let err = repo.get("btop").unwrap_err();
    assert!(matches!(err, RepoError::Package { ref name, .. } if name == "btop"));

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn test_repo_get_name_mismatch() {
    let temp = temp_test_dir("name_mismatch");
    let pkgs_dir = temp.join("pkgs");
    let repo = Repo::new(&pkgs_dir);

    let mismatched = VALID_PACKAGE_TOML.replace("name = \"btop\"", "name = \"curl\"");
    let pkg_file = repo.package_path("btop");
    fs::create_dir_all(pkg_file.parent().unwrap()).unwrap();
    fs::write(&pkg_file, mismatched).unwrap();

    let err = repo.get("btop").unwrap_err();
    assert!(matches!(
        err,
        RepoError::NameMismatch {
            ref expected,
            ref found,
            ..
        } if expected == "btop" && found == "curl"
    ));

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn test_repo_error_display() {
    let not_found = RepoError::NotFound {
        name: "foo".to_string(),
        path: PathBuf::from("/path/to/foo"),
    };
    assert_eq!(
        not_found.to_string(),
        "package `foo` not found at `/path/to/foo`"
    );

    let mismatch = RepoError::NameMismatch {
        expected: "foo".to_string(),
        found: "bar".to_string(),
        path: PathBuf::from("/path/to/package.toml"),
    };
    assert_eq!(
        mismatch.to_string(),
        "package name mismatch at `/path/to/package.toml`: expected `foo`, found `bar`"
    );
}

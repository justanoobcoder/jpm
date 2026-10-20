use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use jpm::consts;
use jpm::package::BuildSystem;
use jpm::repo::{Repo, RepoError, get_package};

fn temp_repo_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("jpm_repo_integration_{label}_{}_{nanos}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

const SAMPLE_MANIFEST: &str = r#"
[info]
name = "nano"
pkgver = "8.1"
pkgrel = 1
desc = "Small, friendly text editor"
license = "GPL-3.0-or-later"
src = "https://www.nano-editor.org/dist/v8/nano-8.1.tar.xz"
hash = "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"

[build]
system = "autotools"
configure = "./configure --prefix=/usr"
compile = "make"
install = "make install"
"#;

#[test]
fn test_integration_read_real_repo_btop() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo = Repo::new(PathBuf::from(manifest_dir).join("pkgs"));

    let pkg = repo.get("btop").expect("real btop package should parse");
    assert_eq!(pkg.info.name, "btop");
    assert_eq!(pkg.info.pkgver, "1.4.7");
    assert_eq!(pkg.info.pkgrel, 1);
    assert_eq!(pkg.info.desc, "A TUI monitor of resources");
    assert_eq!(pkg.info.license, "none");
    assert_eq!(
        pkg.info.url.as_deref(),
        Some("https://github.com/aristocratos/btop")
    );
    assert_eq!(pkg.build.system, BuildSystem::Make);
    assert_eq!(
        pkg.build.install.as_deref(),
        Some("make DESTDIR=$DESTDIR PREFIX=/usr install")
    );
    assert!(pkg.hook.is_some());
}

#[test]
fn test_integration_env_var_override() {
    let temp = temp_repo_dir("env_override");
    let pkgs_dir = temp.join("pkgs");

    let nano_path = pkgs_dir.join("na").join("nano").join("package.toml");
    fs::create_dir_all(nano_path.parent().unwrap()).unwrap();
    fs::write(&nano_path, SAMPLE_MANIFEST).unwrap();

    let prev = std::env::var_os(consts::MANIFEST_DIR_ENV);
    unsafe {
        std::env::set_var(consts::MANIFEST_DIR_ENV, &temp);
    }

    let pkg = get_package("nano").expect("package should be found via JPM_MANIFEST_DIR");
    assert_eq!(pkg.info.name, "nano");
    assert_eq!(pkg.info.pkgver, "8.1");
    assert_eq!(pkg.build.system, BuildSystem::Autotools);

    if let Some(val) = prev {
        unsafe {
            std::env::set_var(consts::MANIFEST_DIR_ENV, val);
        }
    } else {
        unsafe {
            std::env::remove_var(consts::MANIFEST_DIR_ENV);
        }
    }

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn test_integration_package_not_found() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let repo = Repo::new(PathBuf::from(manifest_dir).join("pkgs"));

    let err = repo.get("nonexistent_package").unwrap_err();
    match &err {
        RepoError::NotFound { name, path } => {
            assert_eq!(name, "nonexistent_package");
            assert!(path.ends_with("no/nonexistent_package/package.toml"));
            assert_eq!(
                format!("package `{name}` not found at `{}`", path.display()),
                err.to_string()
            );
        }
        other => panic!("expected NotFound, got: {other:?}"),
    }
}

#[test]
fn test_integration_package_parse_error() {
    let temp = temp_repo_dir("parse_err");
    let repo = Repo::new(temp.join("pkgs"));

    let broken_file = repo.package_path("broken");
    fs::create_dir_all(broken_file.parent().unwrap()).unwrap();
    fs::write(&broken_file, "this is not valid toml").unwrap();

    let err = repo.get("broken").unwrap_err();
    match &err {
        RepoError::Package { name, path, .. } => {
            assert_eq!(name, "broken");
            assert_eq!(path, &broken_file);
            assert!(err.to_string().starts_with("failed to parse package `broken`"));
        }
        other => panic!("expected Package error, got: {other:?}"),
    }

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn test_integration_name_mismatch_error() {
    let temp = temp_repo_dir("name_mismatch");
    let repo = Repo::new(temp.join("pkgs"));

    let nano_file = repo.package_path("nano");
    fs::create_dir_all(nano_file.parent().unwrap()).unwrap();
    let mismatched_manifest = SAMPLE_MANIFEST.replace("name = \"nano\"", "name = \"btop\"");
    fs::write(&nano_file, mismatched_manifest).unwrap();

    let err = repo.get("nano").unwrap_err();
    match &err {
        RepoError::NameMismatch { expected, found, path } => {
            assert_eq!(expected, "nano");
            assert_eq!(found, "btop");
            assert_eq!(path, &nano_file);
            assert!(err.to_string().contains("package name mismatch"));
        }
        other => panic!("expected NameMismatch error, got: {other:?}"),
    }

    let _ = fs::remove_dir_all(&temp);
}

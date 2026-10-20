use std::path::Path;

use crate::package::Build;

use super::error::{FieldError, HashError, NameError, PackageError};
use super::model::{BuildSystem, Package};

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
        .ok_or_else(|| HashError::BadFormat(hash.to_string()))?;

    let &(_, expected_len) = ALGOS
        .iter()
        .find(|(name, _)| *name == algo)
        .ok_or_else(|| {
            let names: Vec<_> = ALGOS.iter().map(|(n, _)| *n).collect();
            HashError::UnsupportedAlgo {
                algo: algo.to_string(),
                supported: names.join(", "),
            }
        })?;

    if !digest.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(HashError::NotHex.into());
    }

    if digest.len() != expected_len {
        return Err(HashError::BadLength {
            algo: algo.to_string(),
            expected_len,
            got_len: digest.len(),
        }
        .into());
    }

    Ok(())
}

fn validate_dependency(field: &'static str, deps: &Vec<String>) -> Result<(), PackageError> {
    for dep in deps {
        validate_name(field, dep)?;
    }

    Ok(())
}

fn validate_patch_path(patch: &str) -> Result<(), PackageError> {
    let path = Path::new(patch);
    if path.is_absolute() {
        return Err(FieldError::Invalid("hook.patches", "must not be an absolute path").into());
    }
    if path.components().any(|c| {
        matches!(
            c,
            std::path::Component::ParentDir | std::path::Component::Prefix(_)
        )
    }) {
        return Err(FieldError::Invalid("hook.patches", "must not contain path traversal").into());
    }
    if patch.starts_with('~') {
        return Err(FieldError::Invalid("hook.patches", "must not start with `~`").into());
    }
    Ok(())
}

fn validate_patch(field: &'static str, patches: &Vec<String>) -> Result<(), PackageError> {
    for p in patches {
        validate_text(field, p)?;
        validate_patch_path(p)?;
    }

    Ok(())
}

fn validate_build(build: &Build) -> Result<(), PackageError> {
    match build.system {
        BuildSystem::Autotools => {}
        BuildSystem::Cmake => {}
        BuildSystem::Make => {
            if build.configure.is_some() {
                return Err(FieldError::Forbidden("build.configure").into());
            }
        }
        BuildSystem::Meson => {
            if build.out_of_src == Some(false) {
                return Err(FieldError::Invalid(
                    "build.out_of_src",
                    "meson does not support in-source builds",
                )
                .into());
            }
        }
        BuildSystem::Custom => {
            if build.install.is_none() {
                return Err(FieldError::Missing("build.install").into());
            }
        }
    }

    for (name, value) in [
        ("build.configure", &build.configure),
        ("build.compile", &build.compile),
        ("build.install", &build.install),
    ] {
        if value.as_ref().is_some_and(|v| v.is_empty()) {
            return Err(FieldError::Empty(name).into());
        }
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

    if info.pkgrel == 0 {
        return Err(FieldError::Invalid("info.pkgrel", "must not be 0").into());
    }

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

    if let Some(hook) = &pkg.hook {
        validate_patch("hook.patches", &hook.patches)?;

        let fields = [
            ("hook.pre_configure", &hook.pre_configure),
            ("hook.pre_install", &hook.pre_install),
            ("hook.post_install", &hook.post_install),
            ("hook.pre_remove", &hook.pre_remove),
            ("hook.post_remove", &hook.post_remove),
        ];

        for (name, value) in fields {
            if value.as_ref().is_some_and(|v| v.is_empty()) {
                return Err(FieldError::Empty(name).into());
            }
        }
    }

    validate_build(&pkg.build)?;

    Ok(())
}

#[cfg(test)]
mod test;

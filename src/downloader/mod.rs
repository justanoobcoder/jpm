use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use digest::{Digest, DynDigest};
use md5::Md5;
use sha1::Sha1;
use sha2::Sha256;

use crate::package::HashError;

fn new_hasher(algo: &str) -> Result<Box<dyn DynDigest>, String> {
    Ok(match algo {
        "md5" => Box::new(Md5::new()),
        "sha1" => Box::new(Sha1::new()),
        "sha256" => Box::new(Sha256::new()),
        other => return Err(format!("unsupported algorithm: {other}")),
    })
}

fn get_filename_from_url(url: &str) -> Option<&str> {
    let path = url.split(['?', '#']).next()?;
    let name = path.rsplit('/').next()?;
    if name.is_empty() { None } else { Some(name) }
}

pub fn download_package(
    url: &str,
    dir: &Path,
    hash: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let resp = ureq::get(url).call()?;
    let mut reader = resp.into_body().into_reader();

    let name = get_filename_from_url(url).ok_or("cannot determine filename from url")?;
    println!("{:?}", dir);
    fs::create_dir_all(dir)?;
    let dest = dir.join(name);

    let tmp = dest.with_extension("part");
    let mut file = File::create(&tmp)?;

    let (algo, digest) = hash
        .split_once(':')
        .ok_or_else(|| HashError::BadFormat(hash.to_string()))?;

    let mut hasher = new_hasher(algo)?;
    let mut buf = [0u8; 64 * 1024];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n])?;
    }
    file.sync_all()?;

    let got = hex::encode(hasher.finalize());
    if got != digest.to_lowercase() {
        fs::remove_file(&tmp)?;
        return Err(format!("checksum mismatch: expected {digest}, got {got}").into());
    }

    fs::rename(&tmp, &dest)?;
    Ok(dest)
}

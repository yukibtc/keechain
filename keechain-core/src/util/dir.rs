// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub const KEECHAIN_EXTENSION: &str = "keechain";
const KEECHAIN_DOT_EXTENSION: &str = ".keechain";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Impossible to get file name")]
    FailedToGetFileName,
}

pub fn get_keychains_list<P>(path: P) -> Result<Vec<String>, Error>
where
    P: AsRef<Path>,
{
    let paths = fs::read_dir(path)?;
    let mut names: Vec<String> = Vec::new();
    for path in paths {
        let path: PathBuf = path?.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(KEECHAIN_DOT_EXTENSION) {
                let splitted: Vec<&str> = name.split(KEECHAIN_DOT_EXTENSION).collect();
                if let Some(value) = splitted.first() {
                    names.push(value.to_string());
                }
            }
        }
    }
    names.sort_by_key(|a| a.to_lowercase());
    Ok(names)
}

pub fn get_keychain_file<P, S>(path: P, name: S) -> Result<PathBuf, Error>
where
    P: AsRef<Path>,
    S: Into<String>,
{
    let mut keychain_file: PathBuf = path.as_ref().join(name.into());
    keychain_file.set_extension(KEECHAIN_EXTENSION);
    Ok(keychain_file)
}

pub fn rename_psbt_to_signed(psbt_file: &mut PathBuf) -> Result<(), Error> {
    if let Some(mut file_name) = psbt_file.file_name().and_then(OsStr::to_str) {
        if let Some(ext) = psbt_file.extension().and_then(OsStr::to_str) {
            let splitted: Vec<&str> = file_name.split(&format!(".{ext}")).collect();
            file_name = match splitted.first() {
                Some(name) => *name,
                None => return Err(Error::FailedToGetFileName),
            }
        }
        psbt_file.set_file_name(&format!("{file_name}-signed.psbt"));
        Ok(())
    } else {
        Err(Error::FailedToGetFileName)
    }
}

// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const KEYCHAIN_EXTENSION: &str = "keechain";
const KEYCHAIN_DOT_EXTENSION: &str = ".keechain";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Impossible to get file name")]
    FailedToGetFileName,
}

pub fn home() -> PathBuf {
    match dirs::home_dir() {
        Some(path) => path,
        None => Path::new("./").to_path_buf(),
    }
}

pub fn keechain() -> Result<PathBuf, Error> {
    Ok(match dirs::home_dir() {
        Some(path) => {
            let path: PathBuf = path.join(".keechain");
            if !path.exists() {
                std::fs::create_dir_all(path.as_path())?;
            }
            path
        }
        None => Path::new("./keechain").to_path_buf(),
    })
}

pub fn keychains() -> Result<PathBuf, Error> {
    let path: PathBuf = keechain()?.join("keychains");
    if !path.exists() {
        std::fs::create_dir_all(path.as_path())?;
    }
    Ok(path)
}

pub fn get_keychains_list() -> Result<Vec<String>, Error> {
    let paths = fs::read_dir(keychains()?)?;
    let mut names: Vec<String> = Vec::new();
    for path in paths {
        let path: PathBuf = path?.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(KEYCHAIN_DOT_EXTENSION) {
                let splitted: Vec<&str> = name.split(KEYCHAIN_DOT_EXTENSION).collect();
                if let Some(value) = splitted.first() {
                    names.push(value.to_string());
                }
            }
        }
    }
    Ok(names)
}

pub fn get_keychain_file<S>(name: S) -> Result<PathBuf, Error>
where
    S: Into<String>,
{
    let mut keychain_file: PathBuf = keychains()?.join(name.into());
    keychain_file.set_extension(KEYCHAIN_EXTENSION);
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

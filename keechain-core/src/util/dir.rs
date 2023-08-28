// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub const KEECHAIN_EXTENSION: &str = "keechain";
pub(crate) const KEECHAIN_DOT_EXTENSION: &str = ".keechain";

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    FailedToGetFileName,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "IO: {e}"),
            Self::FailedToGetFileName => write!(f, "Impossible to get file name"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
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

pub(crate) fn get_keychain_file<P, S>(path: P, name: S) -> Result<PathBuf, Error>
where
    P: AsRef<Path>,
    S: Into<String>,
{
    let mut keychain_file: PathBuf = path.as_ref().join(name.into());
    keychain_file.set_extension(KEECHAIN_EXTENSION);
    Ok(keychain_file)
}

pub fn rename_psbt(psbt_file: &mut PathBuf, finalized: bool) -> Result<(), Error> {
    if let Some(mut file_name) = psbt_file.file_name().and_then(OsStr::to_str) {
        if let Some(ext) = psbt_file.extension().and_then(OsStr::to_str) {
            let splitted: Vec<&str> = file_name.split(&format!(".{ext}")).collect();
            file_name = match splitted.first() {
                Some(name) => *name,
                None => return Err(Error::FailedToGetFileName),
            }
        }

        let mut filename = file_name.to_string();

        for i in 1..u16::MAX {
            let part = if finalized {
                String::from("finalized")
            } else {
                format!("part-{i}")
            };
            filename = format!("{file_name}-{part}.psbt");
            if !Path::new(&filename).exists() {
                break;
            }
        }

        psbt_file.set_file_name(filename);
        Ok(())
    } else {
        Err(Error::FailedToGetFileName)
    }
}

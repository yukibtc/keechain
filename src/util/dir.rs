// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

pub fn home() -> PathBuf {
    match dirs::home_dir() {
        Some(path) => path,
        None => Path::new("./").to_path_buf(),
    }
}

pub fn keechain() -> Result<PathBuf> {
    Ok(match dirs::home_dir() {
        Some(path) => {
            let path: PathBuf = path.join(".keechain");
            if !path.exists() {
                std::fs::create_dir_all(path.clone())?;
            }
            path
        }
        None => Path::new("./keechain").to_path_buf(),
    })
}

pub fn get_keychain_file<S>(name: S) -> Result<PathBuf>
where
    S: Into<String>,
{
    let mut keychain_file: PathBuf = keechain()?.join(name.into());
    keychain_file.set_extension("keechain");
    Ok(keychain_file)
}

pub fn rename_psbt_to_signed(psbt_file: &mut PathBuf) -> Result<()> {
    if let Some(mut file_name) = psbt_file.file_name().and_then(OsStr::to_str) {
        if let Some(ext) = psbt_file.extension().and_then(OsStr::to_str) {
            let splitted: Vec<&str> = file_name.split(&format!(".{}", ext)).collect();
            file_name = match splitted.first() {
                Some(name) => *name,
                None => return Err(anyhow!("Impossible to get file name")),
            }
        }
        psbt_file.set_file_name(&format!("{}-signed.psbt", file_name));
        Ok(())
    } else {
        Err(anyhow!("Impossible to get file name"))
    }
}

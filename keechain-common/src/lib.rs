// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::io::Error;
use std::path::{Path, PathBuf};

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

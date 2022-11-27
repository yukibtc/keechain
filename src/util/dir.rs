// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::{Path, PathBuf};

use anyhow::Result;

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

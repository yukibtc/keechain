// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::{Path, PathBuf};

use anyhow::Result;

pub fn get_directory() -> Result<PathBuf> {
    Ok(match dirs::home_dir() {
        Some(path) => {
            let path: PathBuf = path.join(".keechain");

            // Create directory if not exist
            if !path.exists() {
                std::fs::create_dir_all(path.clone())?;
            }

            path
        }
        None => Path::new("./").to_path_buf(),
    })
}

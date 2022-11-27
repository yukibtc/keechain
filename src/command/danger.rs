// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;

use anyhow::Result;

use super::open;
use crate::types::Seed;
use crate::util::dir;

pub fn view_seed<S, PSW>(file_name: S, get_password: PSW) -> Result<()>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(file_name, get_password)?;
    println!("\n################################################################\n");
    println!("Mnemonic: {}", seed.mnemonic());
    if let Some(passphrase) = seed.passphrase() {
        println!("Passphrase: {}", passphrase);
    }
    println!("Seed (hex format): {}", seed.to_hex());
    println!("\n################################################################\n");
    Ok(())
}

pub fn wipe<S, PSW>(name: S, get_password: PSW) -> Result<()>
where
    S: Into<String> + Clone,
    PSW: FnOnce() -> Result<String>,
{
    let _ = open(name.clone(), get_password)?;
    let keychain_file: PathBuf = dir::get_keychain_file(name)?;
    std::fs::remove_file(keychain_file)?;
    Ok(())
}

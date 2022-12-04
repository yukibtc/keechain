// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;

use crate::command::open;
use crate::core::crypto::aes::Aes256Encryption;
use crate::core::types::Seed;
use crate::core::util::dir;

pub fn rename<S>(name: S, new_name: S) -> Result<()>
where
    S: Into<String>,
{
    let old = dir::get_keychain_file(name)?;
    let new = dir::get_keychain_file(new_name)?;
    fs::rename(old, new)?;
    Ok(())
}

pub fn change_password<S, PSW, NPSW>(
    name: S,
    get_password: PSW,
    get_new_password: NPSW,
) -> Result<()>
where
    S: Into<String> + Clone,
    PSW: FnOnce() -> Result<String>,
    NPSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(name.clone(), get_password)?;
    let keychain_file = dir::get_keychain_file(name)?;
    let new_password: String = get_new_password()?;
    let mut file: File = File::options()
        .create(true)
        .write(true)
        .open(keychain_file)?;
    file.write_all(&seed.encrypt(new_password)?)?;
    Ok(())
}

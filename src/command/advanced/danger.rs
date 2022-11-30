// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use bitcoin::Network;

use crate::command::open;
use crate::types::{Secrets, Seed};
use crate::util::dir;

pub fn view_secrets<S, PSW>(name: S, get_password: PSW, network: Network) -> Result<Secrets>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(name, get_password)?;
    Secrets::new(seed, network)
}

pub fn wipe<S, PSW>(name: S, get_password: PSW) -> Result<()>
where
    S: Into<String> + Clone,
    PSW: FnOnce() -> Result<String>,
{
    let _ = open(name.clone(), get_password)?;
    let keychain_file: PathBuf = dir::get_keychain_file(name)?;
    let mut file: File = File::options()
        .write(true)
        .truncate(true)
        .open(keychain_file.as_path())?;
    file.write_all(&[0u8; 21])?;
    std::fs::remove_file(keychain_file)?;
    Ok(())
}

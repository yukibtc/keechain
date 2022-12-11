// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bitcoin::psbt::PartiallySignedTransaction;
use bitcoin::Network;

use super::open;
use crate::types::{Psbt, Seed};
use crate::util::dir;

pub fn decode_file(psbt_file: PathBuf, network: Network) -> Result<Psbt> {
    if !psbt_file.exists() && !psbt_file.is_file() {
        return Err(anyhow!("PSBT file not found."));
    }

    let mut file: File = File::open(psbt_file)?;
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;

    let psbt: String = base64::encode(content);
    Ok(Psbt::new(
        PartiallySignedTransaction::from_str(&psbt)?,
        network,
    ))
}

pub fn sign_file_from_seed(seed: &Seed, network: Network, psbt_file: PathBuf) -> Result<bool> {
    let mut psbt: Psbt = decode_file(psbt_file.clone(), network)?;
    let finalized: bool = psbt.sign(seed)?;

    if finalized {
        let mut psbt_file = psbt_file;
        dir::rename_psbt_to_signed(&mut psbt_file)?;
        psbt.save_to_file(psbt_file)?;
    }

    Ok(finalized)
}

pub fn sign_file<S, PSW>(
    name: S,
    get_password: PSW,
    network: Network,
    psbt_file: PathBuf,
) -> Result<bool>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(name, get_password)?;
    sign_file_from_seed(&seed, network, psbt_file)
}

// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use bitcoin::Network;

use crate::error::{Error, Result};
use crate::types::{Psbt, Seed};
use crate::util::dir;

pub fn decode_file<P>(path: P, network: Network) -> Result<Psbt>
where
    P: AsRef<Path>,
{
    let psbt_file = path.as_ref();
    if !psbt_file.exists() && !psbt_file.is_file() {
        return Err(Error::Generic("PSBT file not found.".to_string()));
    }
    let mut file: File = File::open(psbt_file)?;
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;
    Psbt::decode(base64::encode(content), network)
}

pub fn sign_file_from_seed<P>(seed: &Seed, network: Network, path: P) -> Result<bool>
where
    P: AsRef<Path>,
{
    let psbt_file = path.as_ref();
    let mut psbt: Psbt = decode_file(psbt_file, network)?;
    let finalized: bool = psbt.sign(seed)?;
    if finalized {
        let mut psbt_file: PathBuf = psbt_file.to_path_buf();
        dir::rename_psbt_to_signed(&mut psbt_file)?;
        psbt.save_to_file(psbt_file)?;
    }
    Ok(finalized)
}

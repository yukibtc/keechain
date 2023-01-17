// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::{Path, PathBuf};

use bitcoin::Network;

use crate::types::{Psbt, Seed};
use crate::util::dir;

pub fn sign_file_from_seed<P>(seed: &Seed, network: Network, path: P) -> crate::Result<bool>
where
    P: AsRef<Path>,
{
    let psbt_file = path.as_ref();
    let mut psbt: Psbt = Psbt::from_file(psbt_file, network)?;
    let finalized: bool = psbt.sign(seed)?;
    if finalized {
        let mut psbt_file: PathBuf = psbt_file.to_path_buf();
        dir::rename_psbt_to_signed(&mut psbt_file)?;
        psbt.save_to_file(psbt_file)?;
    }
    Ok(finalized)
}

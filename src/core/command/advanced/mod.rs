// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use bdk::keys::bip39::Mnemonic;
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::Network;
use secp256k1::Secp256k1;

pub mod danger;

use crate::command::open;
use crate::core::types::{Index, Seed, WordCount};
use crate::core::util::bip::bip32::ToBip32RootKey;
use crate::core::util::bip::bip85::FromBip85;

pub fn derive<S, PSW>(
    name: S,
    get_password: PSW,
    network: Network,
    word_count: WordCount,
    index: Index,
) -> Result<Mnemonic>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(name, get_password)?;
    let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
    let secp = Secp256k1::new();

    Mnemonic::from_bip85(&secp, &root, word_count, index)
}

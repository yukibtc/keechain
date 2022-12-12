// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use bdk::keys::bip39::Mnemonic;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::Network;

pub mod danger;

use crate::error::Result;
use crate::types::{Index, Seed, WordCount};
use crate::util::bip::bip32::Bip32RootKey;
use crate::util::bip::bip85::FromBip85;

pub fn derive(
    seed: Seed,
    network: Network,
    word_count: WordCount,
    index: Index,
) -> Result<Mnemonic> {
    let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
    let secp = Secp256k1::new();
    Mnemonic::from_bip85(&secp, &root, word_count, index)
}

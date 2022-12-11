// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::Result;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, Fingerprint};
use bitcoin::Network;

pub trait Bip32RootKey {
    type Err;
    fn to_bip32_root_key(&self, network: Network) -> Result<ExtendedPrivKey, Self::Err>;
    fn fingerprint(&self, network: Network) -> Result<Fingerprint, Self::Err>;
}

pub fn account_extended_path(
    purpose: u32,
    network: Network,
    account: Option<u32>,
) -> Result<DerivationPath> {
    // Path: m/<purpose>'/<coin>'/<account>'
    let path: Vec<ChildNumber> = vec![
        ChildNumber::from_hardened_idx(purpose)?,
        ChildNumber::from_hardened_idx(if network.eq(&Network::Bitcoin) { 0 } else { 1 })?,
        ChildNumber::from_hardened_idx(account.unwrap_or(0))?,
    ];
    Ok(DerivationPath::from(path))
}

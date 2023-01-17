// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin::util::bip32::{ChildNumber, DerivationPath, Error, ExtendedPrivKey, Fingerprint};
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
) -> Result<DerivationPath, Error> {
    // Path: m/<purpose>'/<coin>'/<account>'
    let path: Vec<ChildNumber> = vec![
        ChildNumber::from_hardened_idx(purpose)?,
        ChildNumber::from_hardened_idx(u32::from(!network.eq(&Network::Bitcoin)))?,
        ChildNumber::from_hardened_idx(account.unwrap_or(0))?,
    ];
    Ok(DerivationPath::from(path))
}

pub fn get_path(
    purpose: u32,
    network: Network,
    account: Option<u32>,
    change: bool,
    index: Option<u32>,
) -> Result<DerivationPath, Error> {
    // Path: m/<purpose>'/<coin>'/<account>'/<change>/<index>
    let base_path = account_extended_path(purpose, network, account)?;
    let path: Vec<ChildNumber> = vec![
        ChildNumber::from_normal_idx(u32::from(change))?,
        ChildNumber::from_normal_idx(index.unwrap_or(0))?,
    ];
    Ok(base_path.extend(path))
}

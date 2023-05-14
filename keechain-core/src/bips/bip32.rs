// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

pub use bitcoin::util::bip32::*;
use bitcoin::Network;

use crate::SECP256K1;

pub trait Bip32 {
    type Err;

    fn to_bip32_root_key(&self, network: Network) -> Result<ExtendedPrivKey, Self::Err>;

    fn fingerprint(&self, network: Network) -> Result<Fingerprint, Self::Err> {
        let root: ExtendedPrivKey = self.to_bip32_root_key(network)?;
        Ok(root.fingerprint(&SECP256K1))
    }
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

pub fn extended_path(
    purpose: u32,
    network: Network,
    account: Option<u32>,
    change: bool,
) -> Result<DerivationPath, Error> {
    // Path: m/<purpose>'/<coin>'/<account>'/<change>
    let base_path = account_extended_path(purpose, network, account)?;
    let path: Vec<ChildNumber> = vec![ChildNumber::from_normal_idx(u32::from(change))?];
    Ok(base_path.extend(path))
}

pub fn get_path(
    purpose: u32,
    network: Network,
    account: Option<u32>,
    change: bool,
    index: Option<u32>,
) -> Result<DerivationPath, Error> {
    // Path: m/<purpose>'/<coin>'/<account>'/<change>/<index>
    let base_path = extended_path(purpose, network, account, change)?;
    let path: Vec<ChildNumber> = vec![ChildNumber::from_normal_idx(index.unwrap_or(0))?];
    Ok(base_path.extend(path))
}

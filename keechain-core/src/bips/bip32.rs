// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! BIP32
//!
//! <https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki>

pub use bdk::bitcoin::bip32::*;
use bdk::bitcoin::secp256k1::{Secp256k1, Signing};
use bdk::bitcoin::Network;

pub trait Bip32 {
    type Err;

    fn to_bip32_root_key(&self, network: Network) -> Result<ExtendedPrivKey, Self::Err>;

    fn to_bip32_root_pubkey<C>(
        &self,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<ExtendedPubKey, Self::Err>
    where
        C: Signing,
    {
        let root: ExtendedPrivKey = self.to_bip32_root_key(network)?;
        Ok(ExtendedPubKey::from_priv(secp, &root))
    }

    fn fingerprint<C>(
        &self,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<Fingerprint, Self::Err>
    where
        C: Signing,
    {
        let root: ExtendedPrivKey = self.to_bip32_root_key(network)?;
        Ok(root.fingerprint(secp))
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
        ChildNumber::from_hardened_idx(match network {
            Network::Bitcoin => 0,
            _ => 1,
        })?,
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
    let path: [ChildNumber; 1] = [ChildNumber::from_normal_idx(u32::from(change))?];
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
    let path: [ChildNumber; 1] = [ChildNumber::from_normal_idx(index.unwrap_or(0))?];
    Ok(base_path.extend(path))
}

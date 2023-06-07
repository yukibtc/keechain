// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::str::FromStr;

use bdk::miniscript::descriptor::{Descriptor, DescriptorKeyParseError, DescriptorPublicKey};
use bitcoin::Network;

use super::{Purpose, Seed};
use crate::bips::bip32::{
    self, Bip32, ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint,
};
use crate::SECP256K1;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    BIP32(#[from] bitcoin::util::bip32::Error),
    #[error(transparent)]
    Miniscript(#[from] bdk::miniscript::Error),
    #[error(transparent)]
    DescriptorKeyParse(#[from] DescriptorKeyParseError),
    #[error("Unsupported derivation path")]
    UnsupportedDerivationPath,
    #[error("Invalid derivation path: purpose not provided")]
    PurposePathNotFound,
    #[error("Invalid derivation path: invalid coin or not provided")]
    CoinPathNotFound,
    #[error("Descriptor not found")]
    DescriptorNotFound,
}

#[derive(Debug, Clone)]
pub struct Descriptors {
    external: HashMap<Purpose, Descriptor<DescriptorPublicKey>>,
    internal: HashMap<Purpose, Descriptor<DescriptorPublicKey>>,
}

impl Descriptors {
    pub fn new(seed: Seed, network: Network, account: Option<u32>) -> Result<Self, Error> {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let root_fingerprint = root.fingerprint(&SECP256K1);

        let paths: Vec<(Purpose, DerivationPath)> = vec![
            (
                Purpose::PKH,
                bip32::account_extended_path(44, network, account)?,
            ),
            (
                Purpose::SHWPKH,
                bip32::account_extended_path(49, network, account)?,
            ),
            (
                Purpose::WPKH,
                bip32::account_extended_path(84, network, account)?,
            ),
            (
                Purpose::TR,
                bip32::account_extended_path(86, network, account)?,
            ),
        ];

        let capacity: usize = paths.len();
        let mut descriptors = Descriptors {
            external: HashMap::with_capacity(capacity),
            internal: HashMap::with_capacity(capacity),
        };

        for (purpose, path) in paths.into_iter() {
            let derived_private_key: ExtendedPrivKey = root.derive_priv(&SECP256K1, &path)?;
            let derived_public_key: ExtendedPubKey =
                ExtendedPubKey::from_priv(&SECP256K1, &derived_private_key);

            descriptors.external.insert(
                purpose,
                typed_descriptor(root_fingerprint, derived_public_key, &path, false)?,
            );
            descriptors.internal.insert(
                purpose,
                typed_descriptor(root_fingerprint, derived_public_key, &path, true)?,
            );
        }

        Ok(descriptors)
    }

    pub fn external(&self) -> Vec<Descriptor<DescriptorPublicKey>> {
        self.external.clone().into_values().collect()
    }

    pub fn internal(&self) -> Vec<Descriptor<DescriptorPublicKey>> {
        self.internal.clone().into_values().collect()
    }

    pub fn get_by_purpose(
        &self,
        purpose: Purpose,
        internal: bool,
    ) -> Result<Descriptor<DescriptorPublicKey>, Error> {
        if internal {
            self.internal
                .get(&purpose)
                .cloned()
                .ok_or(Error::DescriptorNotFound)
        } else {
            self.external
                .get(&purpose)
                .cloned()
                .ok_or(Error::DescriptorNotFound)
        }
    }
}

pub trait ToDescriptor: Bip32
where
    Error: From<<Self as Bip32>::Err>,
{
    fn to_descriptor(
        &self,
        purpose: Purpose,
        account: Option<u32>,
        change: bool,
        network: Network,
    ) -> Result<DescriptorPublicKey, Error> {
        let root: ExtendedPrivKey = self.to_bip32_root_key(network)?;
        let root_fingerprint = root.fingerprint(&SECP256K1);
        let path = bip32::account_extended_path(purpose.as_u32(), network, account)?;
        let derived_private_key: ExtendedPrivKey = root.derive_priv(&SECP256K1, &path)?;
        let derived_public_key: ExtendedPubKey =
            ExtendedPubKey::from_priv(&SECP256K1, &derived_private_key);
        let (_, desc) = descriptor(root_fingerprint, derived_public_key, &path, change)?;
        Ok(desc)
    }

    fn to_typed_descriptor(
        &self,
        purpose: Purpose,
        account: Option<u32>,
        change: bool,
        network: Network,
    ) -> Result<Descriptor<DescriptorPublicKey>, Error> {
        let root: ExtendedPrivKey = self.to_bip32_root_key(network)?;
        let root_fingerprint = root.fingerprint(&SECP256K1);
        let path = bip32::account_extended_path(purpose.as_u32(), network, account)?;
        let derived_private_key: ExtendedPrivKey = root.derive_priv(&SECP256K1, &path)?;
        let derived_public_key: ExtendedPubKey =
            ExtendedPubKey::from_priv(&SECP256K1, &derived_private_key);
        typed_descriptor(root_fingerprint, derived_public_key, &path, change)
    }
}

pub fn descriptor(
    root_fingerprint: Fingerprint,
    pubkey: ExtendedPubKey,
    path: &DerivationPath,
    change: bool,
) -> Result<(ChildNumber, DescriptorPublicKey), Error> {
    let mut iter_path = path.into_iter();

    let purpose: &ChildNumber = match iter_path.next() {
        Some(child) => child,
        None => return Err(Error::PurposePathNotFound),
    };

    let coin: &ChildNumber = match iter_path.next() {
        Some(ChildNumber::Hardened { index: 0 }) => &ChildNumber::Hardened { index: 0 },
        Some(ChildNumber::Hardened { index: 1 }) => &ChildNumber::Hardened { index: 1 },
        _ => return Err(Error::CoinPathNotFound),
    };

    let account: &ChildNumber = match iter_path.next() {
        Some(child) => child,
        None => &ChildNumber::Hardened { index: 0 },
    };

    let desc: String = format!(
        "[{}/{:#}/{:#}/{:#}]{}/{}/*",
        root_fingerprint,
        purpose,
        coin,
        account,
        pubkey,
        i32::from(change)
    );

    Ok((*purpose, DescriptorPublicKey::from_str(&desc)?))
}

pub fn typed_descriptor(
    root_fingerprint: Fingerprint,
    pubkey: ExtendedPubKey,
    path: &DerivationPath,
    change: bool,
) -> Result<Descriptor<DescriptorPublicKey>, Error> {
    let (purpose, desc) = descriptor(root_fingerprint, pubkey, path, change)?;
    match purpose {
        ChildNumber::Hardened { index: 44 } => Ok(Descriptor::new_pkh(desc)),
        ChildNumber::Hardened { index: 49 } => Ok(Descriptor::new_sh_wpkh(desc)?),
        ChildNumber::Hardened { index: 84 } => Ok(Descriptor::new_wpkh(desc)?),
        ChildNumber::Hardened { index: 86 } => Ok(Descriptor::new_tr(desc, None)?),
        _ => Err(Error::UnsupportedDerivationPath),
    }
}

#[cfg(test)]
mod test {
    use bip39::Mnemonic;

    use super::*;

    #[test]
    fn test_seed_to_descriptor() {
        let mnemonic = Mnemonic::from_str("range special tuna oblige own drama trend render harsh army outdoor bulb brisk sing analyst own fork senior stove flash fire bulk umbrella vast").unwrap();
        let seed = Seed::from_mnemonic(mnemonic);

        // Tr
        let desc: DescriptorPublicKey = seed
            .to_descriptor(Purpose::TR, None, false, Network::Bitcoin)
            .unwrap();
        assert_eq!(desc.to_string(), String::from("[91ef223d/86'/0'/0']xpub6CjhhJyrYK83TKQq797CMiNzc4bpoJiYRBeb7iQ99T6dXrEgvg24hDw3ZKDJLNMyiy9Sbwqaw8TtCdaE4xXhnYwy7ptpNVfEAKUCcz8PMtP/0/*"));

        // Wpkh
        let desc: DescriptorPublicKey = seed
            .to_descriptor(Purpose::WPKH, Some(2345), true, Network::Testnet)
            .unwrap();
        assert_eq!(desc.to_string(), String::from("[91ef223d/84'/1'/2345']tpubDCgYuiX1p1eecECkhNc2bLSktmSDoMTj5J3v184ErUXqHTywQ7X5afv51UGfDVSaYzDWvdHhVyJ6UK8fM27EwGByWdczEERfAA9j2nzHUAj/1/*"));
    }

    #[test]
    fn test_seed_to_typed_descriptor() {
        let mnemonic = Mnemonic::from_str("range special tuna oblige own drama trend render harsh army outdoor bulb brisk sing analyst own fork senior stove flash fire bulk umbrella vast").unwrap();
        let seed = Seed::from_mnemonic(mnemonic);

        // Tr
        let desc: Descriptor<DescriptorPublicKey> = seed
            .to_typed_descriptor(Purpose::TR, None, false, Network::Bitcoin)
            .unwrap();
        assert_eq!(desc.to_string(), String::from("tr([91ef223d/86'/0'/0']xpub6CjhhJyrYK83TKQq797CMiNzc4bpoJiYRBeb7iQ99T6dXrEgvg24hDw3ZKDJLNMyiy9Sbwqaw8TtCdaE4xXhnYwy7ptpNVfEAKUCcz8PMtP/0/*)#qkangwzf"));

        // Wpkh
        let desc: Descriptor<DescriptorPublicKey> = seed
            .to_typed_descriptor(Purpose::WPKH, Some(2345), true, Network::Testnet)
            .unwrap();
        assert_eq!(desc.to_string(), String::from("wpkh([91ef223d/84'/1'/2345']tpubDCgYuiX1p1eecECkhNc2bLSktmSDoMTj5J3v184ErUXqHTywQ7X5afv51UGfDVSaYzDWvdHhVyJ6UK8fM27EwGByWdczEERfAA9j2nzHUAj/1/*)#tj43jnd8"));
    }
}

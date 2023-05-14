// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::collections::HashMap;
use std::str::FromStr;

use bdk::miniscript::descriptor::{Descriptor, DescriptorPublicKey};
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
                Self::descriptor(root_fingerprint, derived_public_key, &path, false)?,
            );
            descriptors.internal.insert(
                purpose,
                Self::descriptor(root_fingerprint, derived_public_key, &path, true)?,
            );
        }

        Ok(descriptors)
    }

    fn descriptor(
        root_fingerprint: Fingerprint,
        pubkey: ExtendedPubKey,
        path: &DerivationPath,
        change: bool,
    ) -> Result<Descriptor<DescriptorPublicKey>, Error> {
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

        let descriptor: String = match purpose {
            ChildNumber::Hardened { index: 44 } => format!("pkh({desc})"),
            ChildNumber::Hardened { index: 49 } => format!("sh(wpkh({desc}))"),
            ChildNumber::Hardened { index: 84 } => format!("wpkh({desc})"),
            ChildNumber::Hardened { index: 86 } => format!("tr({desc})"),
            _ => return Err(Error::UnsupportedDerivationPath),
        };

        Ok(Descriptor::from_str(&descriptor)?)
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

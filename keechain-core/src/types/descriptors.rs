// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use bdk::miniscript::descriptor::Descriptor;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{
    ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint,
};
use bitcoin::Network;

use super::Seed;
use crate::util::bip::bip32::{self, Bip32RootKey};

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
}

#[derive(Debug, Clone)]
pub struct Descriptors {
    pub external: Vec<Descriptor<String>>,
    pub internal: Vec<Descriptor<String>>,
}

impl Descriptors {
    pub fn new(seed: Seed, network: Network, account: Option<u32>) -> Result<Self, Error> {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let secp = Secp256k1::new();
        let root_fingerprint = root.fingerprint(&secp);

        let paths: Vec<DerivationPath> = vec![
            bip32::account_extended_path(44, network, account)?,
            bip32::account_extended_path(49, network, account)?,
            bip32::account_extended_path(84, network, account)?,
            bip32::account_extended_path(86, network, account)?,
        ];

        let capacity: usize = paths.len();
        let mut descriptors = Descriptors {
            external: Vec::with_capacity(capacity),
            internal: Vec::with_capacity(capacity),
        };

        for path in paths.iter() {
            let derived_private_key: ExtendedPrivKey = root.derive_priv(&secp, path)?;
            let derived_public_key: ExtendedPubKey =
                ExtendedPubKey::from_priv(&secp, &derived_private_key);

            descriptors.external.push(Self::descriptor(
                root_fingerprint,
                derived_public_key,
                path,
                false,
            )?);
            descriptors.internal.push(Self::descriptor(
                root_fingerprint,
                derived_public_key,
                path,
                true,
            )?);
        }

        Ok(descriptors)
    }

    fn descriptor(
        root_fingerprint: Fingerprint,
        pubkey: ExtendedPubKey,
        path: &DerivationPath,
        change: bool,
    ) -> Result<Descriptor<String>, Error> {
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

        let descriptor: String = format!(
            "[{}/{:#}/{:#}/{:#}]{}/{}/*",
            root_fingerprint,
            purpose,
            coin,
            account,
            pubkey,
            i32::from(change)
        );

        let descriptor: String = match purpose {
            ChildNumber::Hardened { index: 44 } => format!("pkh({})", descriptor),
            ChildNumber::Hardened { index: 49 } => format!("sh(wpkh({}))", descriptor),
            ChildNumber::Hardened { index: 84 } => format!("wpkh({})", descriptor),
            ChildNumber::Hardened { index: 86 } => format!("tr({})", descriptor),
            _ => return Err(Error::UnsupportedDerivationPath),
        };

        Ok(Descriptor::from_str(&descriptor)?)
    }
}

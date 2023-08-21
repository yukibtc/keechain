// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;
use std::collections::HashMap;
use std::str::FromStr;

use bdk::miniscript::descriptor::{Descriptor, DescriptorKeyParseError, DescriptorPublicKey};
use bitcoin::secp256k1::{Secp256k1, Signing};
use bitcoin::Network;

use super::{Purpose, Seed};
use crate::bips::bip32::{
    self, Bip32, ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint,
};

#[derive(Debug)]
pub enum Error {
    BIP32(bip32::Error),
    Miniscript(bdk::miniscript::Error),
    DescriptorKeyParse(DescriptorKeyParseError),
    UnsupportedDerivationPath,
    PurposePathNotFound,
    CoinPathNotFound,
    DescriptorNotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BIP32(e) => write!(f, "BIP32: {e}"),
            Self::Miniscript(e) => write!(f, "Miniscript: {e}"),
            Self::DescriptorKeyParse(e) => write!(f, "Descriptor Key parse: {e}"),
            Self::UnsupportedDerivationPath => write!(f, "Unsupported derivation path"),
            Self::PurposePathNotFound => write!(f, "Invalid derivation path: purpose not provided"),
            Self::CoinPathNotFound => {
                write!(f, "Invalid derivation path: invalid coin or not provided")
            }
            Self::DescriptorNotFound => write!(f, "Descriptor not found"),
        }
    }
}

impl From<bip32::Error> for Error {
    fn from(e: bip32::Error) -> Self {
        Self::BIP32(e)
    }
}

impl From<bdk::miniscript::Error> for Error {
    fn from(e: bdk::miniscript::Error) -> Self {
        Self::Miniscript(e)
    }
}

impl From<DescriptorKeyParseError> for Error {
    fn from(e: DescriptorKeyParseError) -> Self {
        Self::DescriptorKeyParse(e)
    }
}

#[derive(Debug, Clone)]
pub struct Descriptors {
    external: HashMap<Purpose, Descriptor<DescriptorPublicKey>>,
    internal: HashMap<Purpose, Descriptor<DescriptorPublicKey>>,
}

impl Descriptors {
    pub fn new<C>(
        seed: Seed,
        network: Network,
        account: Option<u32>,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Error>
    where
        C: Signing,
    {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let root_fingerprint = root.fingerprint(secp);

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
            let derived_private_key: ExtendedPrivKey = root.derive_priv(secp, &path)?;
            let derived_public_key: ExtendedPubKey =
                ExtendedPubKey::from_priv(secp, &derived_private_key);

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
    fn to_descriptor<C>(
        &self,
        purpose: Purpose,
        account: Option<u32>,
        change: bool,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<DescriptorPublicKey, Error>
    where
        C: Signing,
    {
        let root: ExtendedPrivKey = self.to_bip32_root_key(network)?;
        let root_fingerprint = root.fingerprint(secp);
        let path = bip32::account_extended_path(purpose.as_u32(), network, account)?;
        let derived_private_key: ExtendedPrivKey = root.derive_priv(secp, &path)?;
        let derived_public_key: ExtendedPubKey =
            ExtendedPubKey::from_priv(secp, &derived_private_key);
        let (_, desc) = descriptor(root_fingerprint, derived_public_key, &path, change)?;
        Ok(desc)
    }

    fn to_typed_descriptor<C>(
        &self,
        purpose: Purpose,
        account: Option<u32>,
        change: bool,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<Descriptor<DescriptorPublicKey>, Error>
    where
        C: Signing,
    {
        let root: ExtendedPrivKey = self.to_bip32_root_key(network)?;
        let root_fingerprint = root.fingerprint(secp);
        let path = bip32::account_extended_path(purpose.as_u32(), network, account)?;
        let derived_private_key: ExtendedPrivKey = root.derive_priv(secp, &path)?;
        let derived_public_key: ExtendedPubKey =
            ExtendedPubKey::from_priv(secp, &derived_private_key);
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
        let secp = Secp256k1::new();
        let mnemonic = Mnemonic::from_str("range special tuna oblige own drama trend render harsh army outdoor bulb brisk sing analyst own fork senior stove flash fire bulk umbrella vast").unwrap();
        let seed = Seed::from_mnemonic(mnemonic);

        // Tr
        let desc: DescriptorPublicKey = seed
            .to_descriptor(Purpose::TR, None, false, Network::Bitcoin, &secp)
            .unwrap();
        assert_eq!(desc.to_string(), String::from("[91ef223d/86'/0'/0']xpub6CjhhJyrYK83TKQq797CMiNzc4bpoJiYRBeb7iQ99T6dXrEgvg24hDw3ZKDJLNMyiy9Sbwqaw8TtCdaE4xXhnYwy7ptpNVfEAKUCcz8PMtP/0/*"));

        // Wpkh
        let desc: DescriptorPublicKey = seed
            .to_descriptor(Purpose::WPKH, Some(2345), true, Network::Testnet, &secp)
            .unwrap();
        assert_eq!(desc.to_string(), String::from("[91ef223d/84'/1'/2345']tpubDCgYuiX1p1eecECkhNc2bLSktmSDoMTj5J3v184ErUXqHTywQ7X5afv51UGfDVSaYzDWvdHhVyJ6UK8fM27EwGByWdczEERfAA9j2nzHUAj/1/*"));
    }

    #[test]
    fn test_seed_to_typed_descriptor() {
        let secp = Secp256k1::new();
        let mnemonic = Mnemonic::from_str("range special tuna oblige own drama trend render harsh army outdoor bulb brisk sing analyst own fork senior stove flash fire bulk umbrella vast").unwrap();
        let seed = Seed::from_mnemonic(mnemonic);

        // Tr
        let desc: Descriptor<DescriptorPublicKey> = seed
            .to_typed_descriptor(Purpose::TR, None, false, Network::Bitcoin, &secp)
            .unwrap();
        assert_eq!(desc.to_string(), String::from("tr([91ef223d/86'/0'/0']xpub6CjhhJyrYK83TKQq797CMiNzc4bpoJiYRBeb7iQ99T6dXrEgvg24hDw3ZKDJLNMyiy9Sbwqaw8TtCdaE4xXhnYwy7ptpNVfEAKUCcz8PMtP/0/*)#qkangwzf"));

        // Wpkh
        let desc: Descriptor<DescriptorPublicKey> = seed
            .to_typed_descriptor(Purpose::WPKH, Some(2345), true, Network::Testnet, &secp)
            .unwrap();
        assert_eq!(desc.to_string(), String::from("wpkh([91ef223d/84'/1'/2345']tpubDCgYuiX1p1eecECkhNc2bLSktmSDoMTj5J3v184ErUXqHTywQ7X5afv51UGfDVSaYzDWvdHhVyJ6UK8fM27EwGByWdczEERfAA9j2nzHUAj/1/*)#tj43jnd8"));
    }
}

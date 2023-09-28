// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;

use bdk::bitcoin::secp256k1::{Secp256k1, Signing};
use bdk::bitcoin::Network;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

use super::Descriptors;
use crate::bips::bip32::{self, Bip32, Fingerprint};
use crate::bips::bip39::Mnemonic;
use crate::bips::bip85::{self, Bip85};
use crate::crypto::MultiEncryption;
use crate::types::{Index, Secrets, Seed, WordCount};
use crate::Result;

#[derive(Debug)]
pub enum Error {
    BIP32(bip32::Error),
    BIP85(bip85::Error),
    Descriptors(super::descriptors::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BIP32(e) => write!(f, "BIP32: {e}"),
            Self::BIP85(e) => write!(f, "BIP85: {e}"),
            Self::Descriptors(e) => write!(f, "Descriptors: {e}"),
        }
    }
}

impl From<bip32::Error> for Error {
    fn from(e: bip32::Error) -> Self {
        Self::BIP32(e)
    }
}

impl From<bip85::Error> for Error {
    fn from(e: bip85::Error) -> Self {
        Self::BIP85(e)
    }
}

impl From<super::descriptors::Error> for Error {
    fn from(e: super::descriptors::Error) -> Self {
        Self::Descriptors(e)
    }
}

#[derive(Deserialize)]
struct KeychainIntermediate {
    mnemonic: Mnemonic,
    passphrases: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct Keychain {
    mnemonic: Mnemonic,
    passphrases: Vec<String>,
    #[serde(skip_serializing)]
    pub seed: Seed,
}

impl fmt::Debug for Keychain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<sensitive>")
    }
}

impl Drop for Keychain {
    fn drop(&mut self) {
        self.mnemonic = Mnemonic::from_entropy(b"00000000000000000000000000000000").unwrap();
        self.passphrases = Vec::new();
    }
}

impl<'de> Deserialize<'de> for Keychain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let intermediate = KeychainIntermediate::deserialize(deserializer)?;
        Ok(Self::new(intermediate.mnemonic, intermediate.passphrases))
    }
}

impl Keychain {
    pub fn new(mnemonic: Mnemonic, passphrases: Vec<String>) -> Self {
        Self {
            mnemonic: mnemonic.clone(),
            passphrases,
            seed: Seed::from_mnemonic(mnemonic),
        }
    }

    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic.clone()
    }

    pub fn passphrases(&self) -> Vec<String> {
        self.passphrases.clone()
    }

    pub fn seed(&self) -> Seed {
        self.seed.clone()
    }

    pub fn identity<C>(&self, network: Network, secp: &Secp256k1<C>) -> Result<Fingerprint, Error>
    where
        C: Signing,
    {
        Ok(self.seed.fingerprint(network, secp)?)
    }

    pub fn deterministic_entropy<C>(
        &self,
        word_count: WordCount,
        index: Index,
        secp: &Secp256k1<C>,
    ) -> Result<Mnemonic, Error>
    where
        C: Signing,
    {
        Ok(self.seed.derive_bip85_mnemonic(word_count, index, secp)?)
    }

    pub fn descriptors<C>(
        &self,
        network: Network,
        account: Option<u32>,
        secp: &Secp256k1<C>,
    ) -> Result<Descriptors, Error>
    where
        C: Signing,
    {
        Ok(Descriptors::new(self.seed(), network, account, secp)?)
    }

    pub fn secrets<C>(&self, network: Network, secp: &Secp256k1<C>) -> Result<Secrets, Error>
    where
        C: Signing,
    {
        Ok(Secrets::new(self.seed(), network, secp)?)
    }

    pub fn add_passphrase<S>(&mut self, passphrase: S)
    where
        S: Into<String>,
    {
        let passphrase: String = passphrase.into();
        if !self.passphrases.contains(&passphrase) {
            self.passphrases.push(passphrase);
        }
    }

    pub fn remove_passphrase<S>(&mut self, passphrase: S)
    where
        S: Into<String>,
    {
        let passphrase = passphrase.into();
        if let Some(index) = self.passphrases.iter().position(|p| p == &passphrase) {
            self.remove_passphrase_by_index(index);
        }
    }

    pub fn remove_passphrase_by_index(&mut self, index: usize) {
        self.passphrases.remove(index);
    }

    pub fn get_passphrase(&self, index: usize) -> Option<String> {
        self.passphrases.get(index).cloned()
    }

    pub fn apply_passphrase<S>(&mut self, passphrase: Option<S>)
    where
        S: Into<String>,
    {
        self.seed = Seed::new(self.mnemonic.clone(), passphrase);
    }
}

impl MultiEncryption for Keychain {}

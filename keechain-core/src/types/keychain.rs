// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;

use bitcoin::hashes::Hash;
use bitcoin::Network;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

use super::Descriptors;
use crate::bips::bip32::{self, Bip32, Fingerprint};
use crate::bips::bip39::Mnemonic;
use crate::bips::bip85::{self, Bip85};
use crate::crypto::aes::{self, Aes256Encryption};
use crate::crypto::hash;
use crate::types::{Index, Secrets, Seed, WordCount};
use crate::util;
use crate::Result;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Aes(#[from] aes::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    BIP32(#[from] bip32::Error),
    #[error(transparent)]
    BIP85(#[from] bip85::Error),
    #[error(transparent)]
    Descriptors(#[from] super::descriptors::Error),
    #[error("Impossible to decrypt file: invalid password or content")]
    DecryptionFailed,
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

    pub fn identity(&self, network: Network) -> Result<Fingerprint, Error> {
        Ok(self.seed.fingerprint(network)?)
    }

    pub fn deterministic_entropy(
        &self,
        word_count: WordCount,
        index: Index,
    ) -> Result<Mnemonic, Error> {
        Ok(self.seed.derive_bip85_mnemonic(word_count, index)?)
    }

    pub fn descriptors(
        &self,
        network: Network,
        account: Option<u32>,
    ) -> Result<Descriptors, Error> {
        Ok(Descriptors::new(self.seed(), network, account)?)
    }

    pub fn secrets(&self, network: Network) -> Result<Secrets, Error> {
        Ok(Secrets::new(self.seed(), network)?)
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

impl Aes256Encryption for Keychain {
    type Err = Error;
    fn encrypt<K>(&self, key: K) -> Result<String, Self::Err>
    where
        K: AsRef<[u8]>,
    {
        let serialized: Vec<u8> = util::serde::serialize(self)?;
        let key: [u8; 32] = hash::sha256(key).into_inner();
        Ok(aes::encrypt(key, serialized))
    }

    fn decrypt<K>(key: K, content: &[u8]) -> Result<Self, Self::Err>
    where
        K: AsRef<[u8]>,
    {
        let key: [u8; 32] = hash::sha256(key).into_inner();
        match aes::decrypt(key, content) {
            Ok(data) => Ok(util::serde::deserialize(data)?),
            Err(aes::Error::WrongBlockMode) => Err(Error::DecryptionFailed),
            Err(e) => Err(Error::Aes(e)),
        }
    }
}

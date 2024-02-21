// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;
use core::ops::Deref;

use bdk::bitcoin::secp256k1::{Secp256k1, Signing};
use bdk::bitcoin::Network;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::bips::bip32::{self, Bip32, ExtendedPubKey, Fingerprint};
use crate::bips::bip39::Mnemonic;
use crate::bips::bip85::{self, Bip85};
use crate::crypto::{self, MultiEncryption};
use crate::types::{Index, Secrets, Seed, WordCount};
use crate::{descriptors, Descriptors, Result};

#[derive(Debug)]
pub enum Error {
    BIP32(bip32::Error),
    BIP85(bip85::Error),
    Crypto(crypto::Error),
    Descriptors(descriptors::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BIP32(e) => write!(f, "BIP32: {e}"),
            Self::BIP85(e) => write!(f, "BIP85: {e}"),
            Self::Crypto(e) => write!(f, "Crypto: {e}"),
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

impl From<crypto::Error> for Error {
    fn from(e: crypto::Error) -> Self {
        Self::Crypto(e)
    }
}

impl From<descriptors::Error> for Error {
    fn from(e: descriptors::Error) -> Self {
        Self::Descriptors(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedKeychain {
    pub(crate) master_bip32_root_pubkey: ExtendedPubKey,
    pub(crate) current_bip32_root_pubkey: ExtendedPubKey,
    pub(crate) passphrase: Option<String>,
    pub(crate) raw: String,
    network: Network,
}

impl Deref for EncryptedKeychain {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl EncryptedKeychain {
    pub fn new<S>(bip32_root_pubkey: ExtendedPubKey, keychain: S, network: Network) -> Self
    where
        S: Into<String>,
    {
        Self {
            master_bip32_root_pubkey: bip32_root_pubkey,
            current_bip32_root_pubkey: bip32_root_pubkey,
            passphrase: None,
            raw: keychain.into(),
            network,
        }
    }

    pub fn fingerprint(&self) -> Fingerprint {
        self.current_bip32_root_pubkey.fingerprint()
    }

    /// Check if is using passphrase
    pub fn has_passphrase(&self) -> bool {
        self.passphrase.is_some()
    }

    /// Current passphrase
    pub fn passphrase(&self) -> Option<String> {
        self.passphrase.clone()
    }

    /// Get encrypted keychain data
    pub fn raw(&self) -> String {
        self.raw.clone()
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn keychain<T>(&self, password: T) -> Result<Keychain, Error>
    where
        T: AsRef<[u8]>,
    {
        Ok(Keychain::decrypt(password, self.raw.as_bytes())?)
    }

    pub fn add_passphrase<T, S>(&mut self, password: T, passphrase: S) -> Result<(), Error>
    where
        T: AsRef<[u8]>,
        S: Into<String>,
    {
        let mut keychain: Keychain = self.keychain(&password)?;
        keychain.add_passphrase(passphrase);
        self.raw = keychain.encrypt(password)?;
        Ok(())
    }

    pub fn remove_passphrase<T, S>(&mut self, password: T, passphrase: S) -> Result<(), Error>
    where
        T: AsRef<[u8]>,
        S: Into<String>,
    {
        let mut keychain: Keychain = self.keychain(&password)?;
        keychain.remove_passphrase(passphrase);
        self.raw = keychain.encrypt(password)?;
        Ok(())
    }

    pub fn apply_passphrase<T, S, C>(
        &mut self,
        password: T,
        passphrase: Option<S>,
        secp: &Secp256k1<C>,
    ) -> Result<(), Error>
    where
        T: AsRef<[u8]>,
        S: Into<String>,
        C: Signing,
    {
        let mut keychain: Keychain = self.keychain(&password)?;
        keychain.apply_passphrase(passphrase);
        self.passphrase = keychain.seed.passphrase();
        self.current_bip32_root_pubkey = keychain.seed.to_bip32_root_pubkey(self.network, secp)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct KeychainIntermediate {
    mnemonic: Mnemonic,
    passphrases: Vec<String>,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Keychain {
    mnemonic: Mnemonic,
    passphrases: Vec<String>,
    pub seed: Seed,
}

impl fmt::Debug for Keychain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<sensitive>")
    }
}

impl Deref for Keychain {
    type Target = Seed;
    fn deref(&self) -> &Self::Target {
        &self.seed
    }
}

impl Serialize for Keychain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let intermediate = KeychainIntermediate {
            mnemonic: self.mnemonic.clone(),
            passphrases: self.passphrases.clone(),
        };
        intermediate.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Keychain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let intermediate = KeychainIntermediate::deserialize(deserializer)?;
        Ok(Self::new(
            intermediate.mnemonic.clone(),
            intermediate.passphrases.clone(),
        ))
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
        Ok(Descriptors::new(&self.seed, network, account, secp)?)
    }

    pub fn secrets<C>(&self, network: Network, secp: &Secp256k1<C>) -> Result<Secrets, Error>
    where
        C: Signing,
    {
        Ok(Secrets::new(&self.seed, network, secp)?)
    }

    pub(crate) fn add_passphrase<S>(&mut self, passphrase: S)
    where
        S: Into<String>,
    {
        let passphrase: String = passphrase.into();
        if !self.passphrases.contains(&passphrase) {
            self.passphrases.push(passphrase);
        }
    }

    pub(crate) fn remove_passphrase<S>(&mut self, passphrase: S)
    where
        S: Into<String>,
    {
        let passphrase = passphrase.into();
        if let Some(index) = self.passphrases.iter().position(|p| p == &passphrase) {
            self.passphrases.remove(index);
        }
    }

    pub fn get_passphrase(&self, index: usize) -> Option<String> {
        self.passphrases.get(index).cloned()
    }

    pub(crate) fn apply_passphrase<S>(&mut self, passphrase: Option<S>)
    where
        S: Into<String>,
    {
        self.seed = Seed::new(self.mnemonic.clone(), passphrase);
    }
}

impl MultiEncryption for Keychain {}

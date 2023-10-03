// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;

use keechain_core::types::keychain;
use uniffi::Object;

use super::seed::Seed;
use crate::bips::bip39::Mnemonic;

#[derive(Object)]
pub struct Keychain {
    inner: keychain::Keychain,
}

#[uniffi::export]
impl Keychain {
    #[uniffi::constructor]
    pub fn new(mnemonic: Arc<Mnemonic>, passphrases: Vec<String>) -> Arc<Self> {
        Arc::new(Self {
            inner: keychain::Keychain::new(mnemonic.as_ref().deref().clone(), passphrases),
        })
    }

    pub fn mnemonic(&self) -> Arc<Mnemonic> {
        Arc::new(self.inner.mnemonic().into())
    }

    pub fn passphrases(&self) -> Vec<String> {
        self.inner.passphrases()
    }

    pub fn seed(&self) -> Arc<Seed> {
        Arc::new(self.inner.seed().clone().into())
    }

    /* pub fn identity(&self, network: Network) -> Result<Fingerprint, Error> {
        Ok(self.seed.fingerprint(network)?)
    } */

    /* pub fn deterministic_entropy(
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
    } */

    /* pub fn add_passphrase<S>(&mut self, passphrase: S)
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
    } */
}

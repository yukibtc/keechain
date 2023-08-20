// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;
use std::sync::Arc;

use keechain_core::types::seed;
use keechain_core::bips::bip39::Mnemonic;
use uniffi::Object;

use crate::error::Result;

#[derive(Object)]
pub struct Seed {
    inner: seed::Seed,
}

impl From<seed::Seed> for Seed {
    fn from(inner: seed::Seed) -> Self {
        Self { inner }
    }
}

#[uniffi::export]
impl Seed {
    #[uniffi::constructor]
    pub fn from_mnemonic(mnemonic: String) -> Result<Arc<Self>> {
        let mnemonic = Mnemonic::from_str(&mnemonic)?;
        Ok(Arc::new(Self {
            inner: seed::Seed::from_mnemonic(mnemonic),
        }))
    }

    pub fn mnemonic(&self) -> String {
        self.inner.mnemonic().to_string()
    }

    pub fn passphrase(&self) -> Option<String> {
        self.inner.passphrase()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.inner.to_bytes().to_vec()
    }

    pub fn to_hex(&self) -> String {
        self.inner.to_hex()
    }
}
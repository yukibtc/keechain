// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::Deref;
use std::sync::Arc;
use std::str::FromStr;

use keechain_core::bips::bip39;
use uniffi::Object;

use crate::error::Result;

#[derive(Object)]
pub struct Mnemonic {
    inner: bip39::Mnemonic
}

impl Deref for Mnemonic {
    type Target = bip39::Mnemonic;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[uniffi::export]
impl Mnemonic {
    #[uniffi::constructor]
    pub fn from_string(mnemonic: String) -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            inner: bip39::Mnemonic::from_str(&mnemonic)?
        }))
    }

    pub fn to_str(&self) -> String {
        self.inner.to_string()
    }
}
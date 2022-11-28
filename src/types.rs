// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bdk::keys::bip39::Mnemonic;
use bdk::miniscript::Descriptor;
use bitcoin::util::bip32::ExtendedPrivKey;
use bitcoin::Network;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::util::aes::{self, Aes256Encryption};
use crate::util::bip::bip32::ToBip32RootKey;
use crate::util::{self, convert};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Seed {
    mnemonic: Mnemonic,
    passphrase: Option<String>,
}

impl Seed {
    pub fn new<S>(mnemonic: S, passphrase: Option<S>) -> Result<Self>
    where
        S: Into<String>,
    {
        Ok(Self {
            mnemonic: Mnemonic::from_str(&mnemonic.into())?,
            passphrase: passphrase.map(|p| p.into()),
        })
    }

    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic.clone()
    }

    pub fn passphrase(&self) -> Option<String> {
        self.passphrase.clone()
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        self.mnemonic
            .to_seed(self.passphrase.clone().unwrap_or_default())
    }

    pub fn to_hex(&self) -> String {
        convert::bytes_to_hex_string(self.to_bytes().to_vec())
    }
}

impl ToBip32RootKey for Seed {
    type Err = anyhow::Error;
    fn to_bip32_root_key(&self, network: Network) -> Result<ExtendedPrivKey, Self::Err> {
        Ok(ExtendedPrivKey::new_master(network, &self.to_bytes())?)
    }
}

impl Aes256Encryption for Seed {
    type Err = anyhow::Error;
    fn encrypt<K>(&self, key: K) -> Result<Vec<u8>, Self::Err>
    where
        K: AsRef<[u8]>,
    {
        let serialized_seed: Vec<u8> = util::serialize(self)?;
        Ok(aes::encrypt(key, &serialized_seed))
    }

    fn decrypt<K>(key: K, content: &[u8]) -> Result<Self, Self::Err>
    where
        K: AsRef<[u8]>,
    {
        match aes::decrypt(key, content) {
            Ok(data) => util::deserialize(data),
            Err(aes::Error::WrongBlockMode) => Err(anyhow!(
                "Impossible to decrypt file: invalid password or content"
            )),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
#[repr(u8)]
pub enum WordCount {
    #[clap(name = "12")]
    W12 = 12,
    #[clap(name = "18")]
    W18 = 18,
    #[clap(name = "24")]
    W24 = 24,
}

impl WordCount {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Clone, Copy)]
pub struct Index(u32);

impl Index {
    pub fn new(index: u32) -> Result<Self> {
        if index & (1 << 31) == 0 {
            Ok(Self(index))
        } else {
            Err(anyhow!("Invalid index"))
        }
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl FromStr for Index {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let index: u32 = s.parse()?;
        Self::new(index)
    }
}

impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}

#[derive(Debug, Clone)]
pub struct Descriptors {
    pub external: Vec<Descriptor<String>>,
    pub internal: Vec<Descriptor<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed() {
        let mnemonic: &str = "easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt";
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase).unwrap();
        assert_eq!(&seed.to_hex(), "fb826595a0d679f5e9f8c799bd1decb8dc2ad3fb4e39a1ffaa4708a150e0e81ae55d3f340a188cd6188a2b76601aeae16945b36ae0ecfced9645029796c33713")
    }

    #[test]
    fn test_index() {
        let index = Index::new(2345).unwrap();
        assert_eq!(index.as_u32(), 2345);
        assert!(Index::new(2147483647).is_ok());
        assert!(Index::new(2147483648).is_err());
    }
}

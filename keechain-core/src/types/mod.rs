// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

use bdk::bitcoin::secp256k1::{Secp256k1, Signing};
use bdk::bitcoin::Network;
use bip39::Mnemonic;

pub mod keechain;
pub mod keychain;
pub mod seed;

pub use self::keechain::KeeChain;
pub use self::keychain::{EncryptedKeychain, Keychain};
pub use self::seed::Seed;
use crate::bips::bip32::{self, Bip32, ExtendedPrivKey, Fingerprint};
use crate::util::hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum WordCount {
    W12 = 12,
    W18 = 18,
    W24 = 24,
}

impl Default for WordCount {
    fn default() -> Self {
        Self::W24
    }
}

impl WordCount {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

impl fmt::Display for WordCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}

#[derive(Debug)]
pub enum IndexError {
    ParseInt(ParseIntError),
    InvalidIndex,
}

impl std::error::Error for IndexError {}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseInt(e) => write!(f, "Parse Int: {e}"),
            Self::InvalidIndex => write!(f, "Invalid index"),
        }
    }
}

impl From<ParseIntError> for IndexError {
    fn from(e: ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

#[derive(Clone, Copy, Default)]
pub struct Index(u32);

pub const MAX_INDEX: u32 = 0x80000000;

impl Index {
    pub fn new(index: u32) -> Result<Self, IndexError> {
        if index < MAX_INDEX {
            Ok(Self(index))
        } else {
            Err(IndexError::InvalidIndex)
        }
    }

    pub fn increment(&mut self) {
        if self.0 >= MAX_INDEX {
            self.0 = 0;
        } else {
            self.0 += 1;
        }
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl FromStr for Index {
    type Err = IndexError;
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

#[derive(Clone)]
pub struct Secrets {
    pub entropy: String,
    pub mnemonic: Mnemonic,
    pub passphrase: Option<String>,
    pub seed_hex: String,
    pub network: Network,
    pub root_key: ExtendedPrivKey,
    pub fingerprint: Fingerprint,
}

impl fmt::Debug for Secrets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<sensitive>")
    }
}

impl Secrets {
    pub fn new<C>(seed: &Seed, network: Network, secp: &Secp256k1<C>) -> Result<Self, bip32::Error>
    where
        C: Signing,
    {
        let mnemonic: Mnemonic = seed.mnemonic();
        let root_key: ExtendedPrivKey = seed.to_bip32_root_key(network)?;

        Ok(Self {
            entropy: hex::encode(mnemonic.to_entropy()),
            mnemonic,
            passphrase: seed.passphrase(),
            seed_hex: seed.to_hex(),
            network,
            root_key,
            fingerprint: root_key.fingerprint(secp),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index() {
        let index = Index::new(2345).unwrap();
        assert_eq!(index.as_u32(), 2345);
        assert!(Index::new(2147483647).is_ok());
        assert!(Index::new(2147483648).is_err());
    }
}

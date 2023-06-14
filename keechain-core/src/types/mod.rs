// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::str::FromStr;

use bip39::Mnemonic;
use bitcoin::Network;

pub mod bitcoin_core;
pub mod descriptors;
pub mod electrum;
pub mod keechain;
pub mod keychain;
pub mod psbt;
pub mod seed;
pub mod wasabi;

pub use self::bitcoin_core::BitcoinCore;
pub use self::descriptors::Descriptors;
pub use self::electrum::{Electrum, ElectrumSupportedScripts};
pub use self::keechain::KeeChain;
pub use self::keychain::Keychain;
pub use self::psbt::Psbt;
pub use self::seed::Seed;
pub use self::wasabi::Wasabi;
use crate::bips::bip32::{Bip32, ExtendedPrivKey, Fingerprint};
use crate::util::hex;
use crate::SECP256K1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Purpose {
    PKH = 44,
    SHWPKH = 49,
    WPKH = 84,
    TR = 86,
}

impl Purpose {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

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

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("Invalid index")]
    InvalidIndex,
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
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

pub struct Secrets {
    pub entropy: String,
    pub mnemonic: Mnemonic,
    pub passphrase: Option<String>,
    pub seed_hex: String,
    pub network: Network,
    pub root_key: ExtendedPrivKey,
    pub fingerprint: Fingerprint,
}

impl Secrets {
    pub fn new(seed: Seed, network: Network) -> Result<Self, bitcoin::util::bip32::Error> {
        let mnemonic: Mnemonic = seed.mnemonic();
        let root_key: ExtendedPrivKey = seed.to_bip32_root_key(network)?;

        Ok(Self {
            entropy: hex::encode(mnemonic.to_entropy()),
            mnemonic,
            passphrase: seed.passphrase(),
            seed_hex: seed.to_hex(),
            network,
            root_key,
            fingerprint: root_key.fingerprint(&SECP256K1),
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

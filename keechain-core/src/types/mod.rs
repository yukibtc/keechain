// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::str::FromStr;

use bdk::keys::bip39::Mnemonic;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ExtendedPrivKey, Fingerprint};
use bitcoin::Network;
use clap::ValueEnum;

pub mod bitcoin_core;
pub mod descriptors;
pub mod electrum;
pub mod keychain;
pub mod psbt;
pub mod seed;
pub mod wasabi;

pub use self::bitcoin_core::BitcoinCore;
pub use self::descriptors::Descriptors;
pub use self::electrum::{Electrum, ElectrumSupportedScripts};
pub use self::keychain::{KeeChain, Keychain};
pub use self::psbt::Psbt;
pub use self::seed::Seed;
pub use self::wasabi::Wasabi;
use crate::util::bip::bip32::Bip32RootKey;
use crate::util::convert;

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

#[derive(Debug, thiserror::Error)]
pub enum ErrorIndex {
    #[error("Invalid index")]
    InvalidIndex,
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

#[derive(Clone, Copy, Default)]
pub struct Index(u32);

pub const MAX_INDEX: u32 = 0x80000000;

impl Index {
    pub fn new(index: u32) -> Result<Self, ErrorIndex> {
        if index < MAX_INDEX {
            Ok(Self(index))
        } else {
            Err(ErrorIndex::InvalidIndex)
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
    type Err = ErrorIndex;
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
        let secp = Secp256k1::new();
        let mnemonic: Mnemonic = seed.mnemonic();
        let root_key: ExtendedPrivKey = seed.to_bip32_root_key(network)?;

        Ok(Self {
            entropy: convert::bytes_to_hex(mnemonic.to_entropy()),
            mnemonic,
            passphrase: seed.passphrase(),
            seed_hex: seed.to_hex(),
            network,
            root_key,
            fingerprint: root_key.fingerprint(&secp),
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

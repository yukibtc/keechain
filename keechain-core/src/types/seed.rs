// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;

use bdk::bitcoin::Network;
use bip39::Mnemonic;
use serde::{Deserialize, Serialize};

use crate::bips::bip32::{self, Bip32, ExtendedPrivKey};
use crate::bips::bip85::Bip85;
use crate::descriptors::ToDescriptor;
use crate::util::hex;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Seed {
    mnemonic: Mnemonic,
    passphrase: Option<String>,
}

impl fmt::Debug for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<sensitive>")
    }
}

impl Drop for Seed {
    fn drop(&mut self) {
        self.mnemonic = Mnemonic::from_entropy(b"00000000000000000000000000000000").unwrap();
        self.passphrase = None;
    }
}

impl Seed {
    pub fn new<S>(mnemonic: Mnemonic, passphrase: Option<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            mnemonic,
            passphrase: passphrase.map(|p| p.into()),
        }
    }

    pub fn from_mnemonic(mnemonic: Mnemonic) -> Self {
        Self {
            mnemonic,
            passphrase: None,
        }
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
        hex::encode(self.to_bytes())
    }
}

impl Bip32 for Seed {
    type Err = bip32::Error;
    fn to_bip32_root_key(&self, network: Network) -> Result<ExtendedPrivKey, Self::Err> {
        ExtendedPrivKey::new_master(network, &self.to_bytes())
    }
}

impl Bip85 for Seed {}
impl ToDescriptor for Seed {}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_seed() {
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);
        assert_eq!(&seed.to_hex(), "fb826595a0d679f5e9f8c799bd1decb8dc2ad3fb4e39a1ffaa4708a150e0e81ae55d3f340a188cd6188a2b76601aeae16945b36ae0ecfced9645029796c33713")
    }
}

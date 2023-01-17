// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use bdk::keys::bip39::Mnemonic;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ExtendedPrivKey, Fingerprint};
use bitcoin::Network;
use serde::{Deserialize, Serialize};

use crate::util::bip::bip32::Bip32RootKey;
use crate::util::convert;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Seed {
    mnemonic: Mnemonic,
    passphrase: Option<String>,
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
        convert::bytes_to_hex(self.to_bytes().to_vec())
    }
}

impl Bip32RootKey for Seed {
    type Err = bitcoin::util::bip32::Error;
    fn to_bip32_root_key(&self, network: Network) -> Result<ExtendedPrivKey, Self::Err> {
        ExtendedPrivKey::new_master(network, &self.to_bytes())
    }

    fn fingerprint(&self, network: Network) -> Result<Fingerprint, Self::Err> {
        let root: ExtendedPrivKey = self.to_bip32_root_key(network)?;
        let secp = Secp256k1::new();
        Ok(root.fingerprint(&secp))
    }
}

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

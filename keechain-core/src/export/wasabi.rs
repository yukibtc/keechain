// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use bdk::bitcoin::secp256k1::{Secp256k1, Signing};
use bdk::bitcoin::Network;
use serde::{Deserialize, Serialize};

use crate::bips::bip32::{
    self, Bip32, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint,
};
use crate::types::Seed;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    BIP32(bip32::Error),
    Json(serde_json::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "IO: {e}"),
            Self::BIP32(e) => write!(f, "BIP32: {e}"),
            Self::Json(e) => write!(f, "Json: {e}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<bip32::Error> for Error {
    fn from(e: bip32::Error) -> Self {
        Self::BIP32(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Wasabi {
    #[serde(rename = "ExtPubKey")]
    xpub: ExtendedPubKey,
    #[serde(rename = "MasterFingerprint")]
    root_fingerprint: Fingerprint,
}

impl Wasabi {
    pub fn new<C>(seed: &Seed, network: Network, secp: &Secp256k1<C>) -> Result<Self, Error>
    where
        C: Signing,
    {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let path: DerivationPath = bip32::account_extended_path(84, network, None)?;
        let xpriv: ExtendedPrivKey = root.derive_priv(secp, &path)?;
        let pubkey: ExtendedPubKey = ExtendedPubKey::from_priv(secp, &xpriv);

        Ok(Self {
            xpub: pubkey,
            root_fingerprint: root.fingerprint(secp),
        })
    }

    pub fn as_json(&self) -> String {
        serde_json::json!(self).to_string()
    }

    pub fn save_to_file<P>(&self, path: P) -> Result<PathBuf, Error>
    where
        P: AsRef<Path>,
    {
        let file_name: String = format!("keechain-wasabi-{}.json", self.xpub.fingerprint());
        let path: PathBuf = path.as_ref().join(file_name);
        let mut file: File = File::options().create(true).write(true).open(&path)?;
        file.write_all(&serde_json::to_vec(self)?)?;
        Ok(path)
    }
}

// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use bitcoin::Network;
use serde::{Deserialize, Serialize};

use crate::bips::bip32::{
    self, Bip32, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint,
};
use crate::types::Seed;
use crate::SECP256K1;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    BIP32(#[from] bip32::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Wasabi {
    #[serde(rename = "ExtPubKey")]
    xpub: ExtendedPubKey,
    #[serde(rename = "MasterFingerprint")]
    root_fingerprint: Fingerprint,
}

impl Wasabi {
    pub fn new(seed: Seed, network: Network) -> Result<Self, Error> {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let path: DerivationPath = bip32::account_extended_path(84, network, None)?;
        let xpriv: ExtendedPrivKey = root.derive_priv(&SECP256K1, &path)?;
        let pubkey: ExtendedPubKey = ExtendedPubKey::from_priv(&SECP256K1, &xpriv);

        Ok(Self {
            xpub: pubkey,
            root_fingerprint: root.fingerprint(&SECP256K1),
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

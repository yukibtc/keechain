// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint};
use bitcoin::Network;

use serde::{Deserialize, Serialize};

use crate::bips::bip32::{self, Bip32RootKey};
use crate::slips::slip132::{self, ToSlip132};
use crate::types::Seed;
use crate::SECP256K1;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    BIP32(#[from] bitcoin::util::bip32::Error),
    #[error(transparent)]
    SLIP32(#[from] slip132::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ElectrumSupportedScripts {
    /// P2PKH (BIP44)
    Legacy = 44,
    /// P2SHWPKH (BIP49)
    Segwit = 49,
    /// P2WPKH (BIP84)
    NativeSegwit = 84,
}

impl Default for ElectrumSupportedScripts {
    fn default() -> Self {
        Self::NativeSegwit
    }
}

impl ElectrumSupportedScripts {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

impl fmt::Display for ElectrumSupportedScripts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Legacy => write!(f, "legacy"),
            Self::Segwit => write!(f, "segwit"),
            Self::NativeSegwit => write!(f, "native-segwit"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ElectrumKeystore {
    xpub: String,
    #[serde(skip)]
    fingerprint: Fingerprint,
    root_fingerprint: Fingerprint,
    #[serde(rename = "type")]
    keystore_type: String,
    derivation: DerivationPath,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Electrum {
    keystore: ElectrumKeystore,
    wallet_type: String,
    use_encryption: bool,
    seed_version: u32,
}

impl Electrum {
    pub fn new(
        seed: Seed,
        network: Network,
        script: ElectrumSupportedScripts,
        account: Option<u32>,
    ) -> Result<Self, Error> {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let path: DerivationPath = bip32::account_extended_path(script.as_u32(), network, account)?;
        let xpriv: ExtendedPrivKey = root.derive_priv(&SECP256K1, &path)?;
        let pubkey: ExtendedPubKey = ExtendedPubKey::from_priv(&SECP256K1, &xpriv);

        Ok(Self {
            keystore: ElectrumKeystore {
                xpub: pubkey.to_slip132(&path)?,
                fingerprint: pubkey.fingerprint(),
                root_fingerprint: root.fingerprint(&SECP256K1),
                keystore_type: String::from("bip32"),
                derivation: path,
            },
            wallet_type: String::from("standard"),
            use_encryption: false,
            seed_version: 20,
        })
    }

    pub fn save_to_file<P>(&self, path: P) -> Result<PathBuf, Error>
    where
        P: AsRef<Path>,
    {
        let file_name: String = format!("keechain-{}.json", self.keystore.fingerprint);
        let path: PathBuf = path.as_ref().join(file_name);
        let mut file: File = File::options().create(true).write(true).open(&path)?;
        file.write_all(&serde_json::to_vec(self)?)?;
        Ok(path)
    }
}

// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use bitcoin::secp256k1::{Secp256k1, Signing};
use bitcoin::Network;
use serde::{Deserialize, Serialize};

use crate::bips::bip32::{
    self, Bip32, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint,
};
use crate::slips::slip132::{self, ToSlip132};
use crate::types::Seed;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    BIP32(bip32::Error),
    SLIP32(slip132::Error),
    Json(serde_json::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "IO: {e}"),
            Self::BIP32(e) => write!(f, "BIP32: {e}"),
            Self::SLIP32(e) => write!(f, "SLIP32: {e}"),
            Self::Json(e) => write!(f, "Json: {e}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<slip132::Error> for Error {
    fn from(e: slip132::Error) -> Self {
        Self::SLIP32(e)
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
    pub fn new<C>(
        seed: Seed,
        network: Network,
        script: ElectrumSupportedScripts,
        account: Option<u32>,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Error>
    where
        C: Signing,
    {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let path: DerivationPath = bip32::account_extended_path(script.as_u32(), network, account)?;
        let xpriv: ExtendedPrivKey = root.derive_priv(secp, &path)?;
        let pubkey: ExtendedPubKey = ExtendedPubKey::from_priv(secp, &xpriv);

        Ok(Self {
            keystore: ElectrumKeystore {
                xpub: pubkey.to_slip132(&path)?,
                fingerprint: pubkey.fingerprint(),
                root_fingerprint: root.fingerprint(secp),
                keystore_type: String::from("bip32"),
                derivation: path,
            },
            wallet_type: String::from("standard"),
            use_encryption: false,
            seed_version: 20,
        })
    }

    pub fn as_json(&self) -> String {
        serde_json::json!(self).to_string()
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

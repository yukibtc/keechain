// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::fs::File;
use std::io::Write;

use anyhow::Result;

use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey};
use bitcoin::Network;
use clap::ValueEnum;
use secp256k1::Secp256k1;
use serde_json::json;

use super::extended_private_key;
use crate::util::dir;
use crate::util::slip::slip132::ToSlip132;

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
#[repr(u8)]
pub enum ElectrumExportSupportedScripts {
    /// P2PKH (BIP44)
    Legacy = 44,
    /// P2SHWPKH (BIP49)
    Segwit = 49,
    /// P2WPKH (BIP84)
    NativeSegwit = 84,
}

impl ElectrumExportSupportedScripts {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

impl fmt::Display for ElectrumExportSupportedScripts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Legacy => write!(f, "legacy"),
            Self::Segwit => write!(f, "segwit"),
            Self::NativeSegwit => write!(f, "native-segwit"),
        }
    }
}

/* pub fn bitcoin_core<S>(
    file_name: S,
    password: S,
    network: Network,
    account: Option<u32>,
) -> Result<()>
where
    S: Into<String>,
{
    let descriptors_external = json!({
        "timestamp": "now",
        "label": "Keechain",
        "active": true,
        "desc": "",
        "internal": false,
    });

    let descriptors_internal = json!({
        "timestamp": "now",
        "label": "Keechain",
        "active": true,
        "desc": "",
        "internal": true,
    });

    let descriptors = json!([descriptors_external, descriptors_internal]);

    println!("importdescriptor '{}'", descriptors);

    Ok(())
} */

pub fn electrum<S, PSW>(
    name: S,
    get_password: PSW,
    network: Network,
    path: DerivationPath,
) -> Result<()>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let root: ExtendedPrivKey = extended_private_key(name, get_password, network)?;
    let secp = Secp256k1::new();
    let fingerprint = root.fingerprint(&secp);

    let pubkey = ExtendedPubKey::from_priv(&secp, &root.derive_priv(&secp, &path)?);

    let electrum_json = json!({
        "keystore": {
            "xpub": pubkey.to_slip132(&path)?,
            "root_fingerprint": fingerprint.to_string(),
            "type": "bip32",
            "derivation": path.to_string()
        },
        "wallet_type": "standard",
        "use_encryption": false,
        "seed_version": 48
    });

    let home_dir = dir::home();
    let file_name = format!("keechain-{}.json", pubkey.fingerprint());
    let mut file: File = File::options()
        .create(true)
        .write(true)
        .open(home_dir.join(file_name))?;
    file.write_all(electrum_json.to_string().as_bytes())?;
    println!("New electrum file exported.");

    Ok(())
}

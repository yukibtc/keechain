// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;

use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint};
use bitcoin::Network;
use clap::ValueEnum;
use secp256k1::Secp256k1;
use serde_json::{json, Value};

use super::{descriptor, open};
use crate::types::{Descriptors, Seed};
use crate::util::bip::bip32::{self, ToBip32RootKey};
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

pub fn descriptors<S, PSW>(
    name: S,
    get_password: PSW,
    network: Network,
    account: Option<u32>,
) -> Result<Descriptors>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(name, get_password)?;
    let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
    let secp = Secp256k1::new();
    let root_fingerprint = root.fingerprint(&secp);

    let paths: Vec<DerivationPath> = vec![
        bip32::account_extended_path(44, network, account)?,
        bip32::account_extended_path(49, network, account)?,
        bip32::account_extended_path(84, network, account)?,
        bip32::account_extended_path(86, network, account)?,
    ];

    let mut descriptors = Descriptors {
        external: Vec::new(),
        internal: Vec::new(),
    };

    for path in paths.iter() {
        let derived_private_key: ExtendedPrivKey = root.derive_priv(&secp, path)?;
        let derived_public_key: ExtendedPubKey =
            ExtendedPubKey::from_priv(&secp, &derived_private_key);

        descriptors.external.push(descriptor(
            root_fingerprint,
            derived_public_key,
            path,
            false,
        )?);
        descriptors.internal.push(descriptor(
            root_fingerprint,
            derived_public_key,
            path,
            true,
        )?);
    }

    Ok(descriptors)
}

pub fn bitcoin_core<S, PSW>(
    name: S,
    get_password: PSW,
    network: Network,
    account: Option<u32>,
) -> Result<String>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let descriptors: Descriptors = descriptors(name, get_password, network, account)?;
    let mut bitcoin_core_descriptors: Vec<Value> = Vec::new();

    for external in descriptors.external.iter() {
        bitcoin_core_descriptors.push(json!({
            "timestamp": "now",
            "active": true,
            "desc": external,
            "internal": false,
        }));
    }

    for internal in descriptors.internal.iter() {
        bitcoin_core_descriptors.push(json!({
            "timestamp": "now",
            "active": true,
            "desc": internal,
            "internal": true,
        }));
    }

    Ok(format!(
        "\nimportdescriptors '{}'\n",
        json!(bitcoin_core_descriptors)
    ))
}

pub fn electrum<S, PSW>(
    name: S,
    get_password: PSW,
    network: Network,
    path: DerivationPath,
) -> Result<PathBuf>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(name, get_password)?;
    let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
    let secp = Secp256k1::new();
    let fingerprint: Fingerprint = root.fingerprint(&secp);

    let pubkey: ExtendedPubKey = ExtendedPubKey::from_priv(&secp, &root.derive_priv(&secp, &path)?);

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

    let home_dir: PathBuf = dir::home();
    let file_name: String = format!("keechain-{}.json", pubkey.fingerprint());
    let path: PathBuf = home_dir.join(file_name);
    let mut file: File = File::options().create(true).write(true).open(&path)?;
    file.write_all(electrum_json.to_string().as_bytes())?;
    Ok(path)
}

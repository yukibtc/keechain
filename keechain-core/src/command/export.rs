// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use bdk::miniscript::descriptor::Descriptor;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{
    ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint,
};
use bitcoin::Network;
use serde_json::json;

use crate::error::{Error, Result};
use crate::types::{BitcoinCoreDescriptor, Descriptors, ElectrumExportSupportedScripts, Seed};
use crate::util::bip::bip32::{self, Bip32RootKey};
use crate::util::dir;
use crate::util::slip::slip132::ToSlip132;

pub fn descriptor(
    root_fingerprint: Fingerprint,
    pubkey: ExtendedPubKey,
    path: &DerivationPath,
    change: bool,
) -> Result<Descriptor<String>> {
    let mut iter_path = path.into_iter();

    let purpose: &ChildNumber = match iter_path.next() {
        Some(child) => child,
        None => {
            return Err(Error::Generic(
                "Invalid derivation path: purpose not provided".to_string(),
            ))
        }
    };

    let coin: &ChildNumber = match iter_path.next() {
        Some(ChildNumber::Hardened { index: 0 }) => &ChildNumber::Hardened { index: 0 },
        Some(ChildNumber::Hardened { index: 1 }) => &ChildNumber::Hardened { index: 1 },
        _ => {
            return Err(Error::Generic(
                "Invalid derivation path: coin invalid or not provided".to_string(),
            ))
        }
    };

    let account: &ChildNumber = match iter_path.next() {
        Some(child) => child,
        None => &ChildNumber::Hardened { index: 0 },
    };

    let descriptor: String = format!(
        "[{}/{:#}/{:#}/{:#}]{}/{}/*",
        root_fingerprint,
        purpose,
        coin,
        account,
        pubkey,
        i32::from(change)
    );

    let descriptor: String = match purpose {
        ChildNumber::Hardened { index: 44 } => format!("pkh({})", descriptor),
        ChildNumber::Hardened { index: 49 } => format!("sh(wpkh({}))", descriptor),
        ChildNumber::Hardened { index: 84 } => format!("wpkh({})", descriptor),
        ChildNumber::Hardened { index: 86 } => format!("tr({})", descriptor),
        _ => return Err(Error::Generic("Unsupported derivation path".to_string())),
    };

    Descriptor::from_str(&descriptor)
        .map_err(|e| Error::Parse(format!("Impossible to parse descriptor: {}", e)))
}

pub fn descriptors(seed: Seed, network: Network, account: Option<u32>) -> Result<Descriptors> {
    let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
    let secp = Secp256k1::new();
    let root_fingerprint = root.fingerprint(&secp);

    let paths: Vec<DerivationPath> = vec![
        bip32::account_extended_path(44, network, account)?,
        bip32::account_extended_path(49, network, account)?,
        bip32::account_extended_path(84, network, account)?,
        bip32::account_extended_path(86, network, account)?,
    ];

    let capacity: usize = paths.len();
    let mut descriptors = Descriptors {
        external: Vec::with_capacity(capacity),
        internal: Vec::with_capacity(capacity),
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

pub fn bitcoin_core(seed: Seed, network: Network, account: Option<u32>) -> Result<String> {
    let descriptors: Descriptors = descriptors(seed, network, account)?;
    let mut bitcoin_core_descriptors: Vec<BitcoinCoreDescriptor> = Vec::new();

    for desc in descriptors.external.into_iter() {
        bitcoin_core_descriptors.push(BitcoinCoreDescriptor::new(desc, false));
    }

    for desc in descriptors.internal.into_iter() {
        bitcoin_core_descriptors.push(BitcoinCoreDescriptor::new(desc, true));
    }

    Ok(format!(
        "\nimportdescriptors '{}'\n",
        json!(bitcoin_core_descriptors)
    ))
}

pub fn electrum(
    seed: Seed,
    network: Network,
    script: ElectrumExportSupportedScripts,
    account: Option<u32>,
) -> Result<PathBuf> {
    let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
    let secp = Secp256k1::new();
    let fingerprint: Fingerprint = root.fingerprint(&secp);
    let path: DerivationPath = bip32::account_extended_path(script.as_u32(), network, account)?;
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

    // TODO: let the user choose the path (by default use home dir if user not provide a path)
    let home_dir: PathBuf = dir::home();
    let file_name: String = format!("keechain-{}.json", pubkey.fingerprint());
    let path: PathBuf = home_dir.join(file_name);
    let mut file: File = File::options().create(true).write(true).open(&path)?;
    file.write_all(electrum_json.to_string().as_bytes())?;
    Ok(path)
}

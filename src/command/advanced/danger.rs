// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use bitcoin::util::bip32::{ExtendedPrivKey, Fingerprint};
use bitcoin::Network;
use prettytable::{row, Table};
use secp256k1::Secp256k1;

use crate::command::open;
use crate::types::Seed;
use crate::util::bip::bip32::ToBip32RootKey;
use crate::util::{convert, dir};

pub fn view_secrets<S, PSW>(name: S, get_password: PSW, network: Network) -> Result<()>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(name, get_password)?;
    let mnemonic = seed.mnemonic();
    let entropy = convert::bytes_to_hex_string(mnemonic.to_entropy());
    let secp = Secp256k1::new();
    let root_key: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
    let fingerprint: Fingerprint = root_key.fingerprint(&secp);

    let mut table = Table::new();

    table.add_row(row![
        format!("Entropy ({} bits)", entropy.len() / 2 * 8),
        entropy
    ]);
    table.add_row(row!["Mnemonic (BIP39)", mnemonic]);

    if let Some(passphrase) = seed.passphrase() {
        table.add_row(row!["Passphrase (BIP39)", passphrase]);
    }

    table.add_row(row!["Seed HEX (BIP39)", seed.to_hex()]);
    table.add_row(row!["Network", network]);
    table.add_row(row!["Root Key (BIP32)", root_key]);
    table.add_row(row!["Fingerprint (BIP32)", fingerprint]);

    table.printstd();
    Ok(())
}

pub fn wipe<S, PSW>(name: S, get_password: PSW) -> Result<()>
where
    S: Into<String> + Clone,
    PSW: FnOnce() -> Result<String>,
{
    let _ = open(name.clone(), get_password)?;
    let keychain_file: PathBuf = dir::get_keychain_file(name)?;
    let mut file: File = File::options()
        .write(true)
        .truncate(true)
        .open(keychain_file.as_path())?;
    file.write_all(&[0u8; 21])?;
    std::fs::remove_file(keychain_file)?;
    Ok(())
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;

use anyhow::Result;
use bitcoin::Network;

use super::open;
use crate::types::Seed;
use crate::util::bip::bip32::ToBip32RootKey;
use crate::util::{convert, dir};

pub fn view_seed<S, PSW>(file_name: S, get_password: PSW, network: Network) -> Result<()>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(file_name, get_password)?;
    let mnemonic = seed.mnemonic();
    let entropy = convert::bytes_to_hex_string(mnemonic.to_entropy());
    println!("\n################################################################\n");
    println!("Entropy: {} ({} bits)", entropy, entropy.len() / 2 * 8);
    println!("BIP39 Mnemonic: {}", mnemonic);
    if let Some(passphrase) = seed.passphrase() {
        println!("BIP39 Passphrase: {}", passphrase);
    }
    println!("BIP39 Seed (hex): {}", seed.to_hex());
    println!("BIP32 Root Key: {}", seed.to_bip32_root_key(network)?);
    println!("\n################################################################\n");
    Ok(())
}

pub fn wipe<S, PSW>(name: S, get_password: PSW) -> Result<()>
where
    S: Into<String> + Clone,
    PSW: FnOnce() -> Result<String>,
{
    let _ = open(name.clone(), get_password)?;
    let keychain_file: PathBuf = dir::get_keychain_file(name)?;
    std::fs::remove_file(keychain_file)?;
    Ok(())
}

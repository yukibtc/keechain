// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bdk::database::MemoryDatabase;
use bdk::keys::bip39::Mnemonic;
use bdk::keys::DescriptorSecretKey;
use bdk::miniscript::Descriptor;
use bdk::wallet::AddressIndex;
use bdk::{descriptor, SignOptions, Wallet};
use bitcoin::psbt::PartiallySignedTransaction;
use bitcoin::util::bip32::{
    ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint,
};
use bitcoin::Network;
use secp256k1::Secp256k1;

pub mod danger;
pub mod export;

use crate::types::{Index, Seed, WordCount};
use crate::util::aes::Aes256Encryption;
use crate::util::bip::bip85::FromBip85;
use crate::util::dir;

pub fn restore<S, PSW, M, P>(
    name: S,
    get_password: PSW,
    get_mnemonic: M,
    get_passphrase: P,
) -> Result<()>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
    M: FnOnce() -> Result<String>,
    P: FnOnce() -> Result<Option<String>>,
{
    let keychain_file: PathBuf = dir::get_keychain_file(name)?;
    if keychain_file.exists() {
        return Err(anyhow!(
            "There is already a file with the same name! Please, choose another name."
        ));
    }

    let password: String = get_password()?;
    if password.is_empty() {
        return Err(anyhow!("Invalid password"));
    }

    let mnemonic: String = get_mnemonic()?;
    let passphrase: Option<String> = get_passphrase()?;

    let seed = Seed::new(mnemonic, passphrase)?;

    let mut file: File = File::options()
        .create_new(true)
        .write(true)
        .open(keychain_file)?;
    file.write_all(&seed.encrypt(password)?)?;

    Ok(())
}

pub fn open<S, PSW>(name: S, get_password: PSW) -> Result<Seed>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let keychain_file: PathBuf = dir::get_keychain_file(name)?;

    // Check if mnemonic file exist
    if !keychain_file.exists() {
        return Err(anyhow!("File not found."));
    }

    // Read seed from file
    let mut file: File = File::open(keychain_file)?;
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;

    let password: String = get_password()?;

    Seed::decrypt(password, &content)
}

pub fn extended_private_key<S, PSW>(
    name: S,
    get_password: PSW,
    network: Network,
) -> Result<ExtendedPrivKey>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(name, get_password)?;
    Ok(ExtendedPrivKey::new_master(network, &seed.to_bytes())?)
}

pub fn account_extended_derivation_path(
    purpose: u32,
    network: Network,
    account: Option<u32>,
) -> Result<DerivationPath> {
    // Path: m/<purpose>'/<coin>'/<account>'
    let path: Vec<ChildNumber> = vec![
        ChildNumber::from_hardened_idx(purpose)?,
        ChildNumber::from_hardened_idx(if network.eq(&Network::Bitcoin) { 0 } else { 1 })?,
        ChildNumber::from_hardened_idx(account.unwrap_or(0))?,
    ];
    Ok(DerivationPath::from(path))
}

fn descriptor(
    pubkey: ExtendedPubKey,
    network: Network,
    purpose: u8,
    account: Option<u32>,
    change: bool,
) -> Result<Descriptor<String>> {
    let descriptor: String = format!(
        "[{}/{}'/{}'/{}']{}/{}/*",
        pubkey.fingerprint(),
        purpose,
        if network.eq(&Network::Bitcoin) { 0 } else { 1 },
        account.unwrap_or(0),
        pubkey,
        i32::from(change)
    );
    match purpose {
        44 => Ok(Descriptor::from_str(&format!("pkh({})", descriptor))?),
        49 => Ok(Descriptor::from_str(&format!("sh(wpkh({}))", descriptor))?),
        84 => Ok(Descriptor::from_str(&format!("wpkh({})", descriptor))?),
        86 => Ok(Descriptor::from_str(&format!("tr({})", descriptor))?),
        _ => Err(anyhow!("Unsupported purpose")),
    }
}

pub fn get_public_keys<S, PSW>(
    name: S,
    get_password: PSW,
    network: Network,
    account: Option<u32>,
) -> Result<()>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let root: ExtendedPrivKey = extended_private_key(name, get_password, network)?;
    let secp = Secp256k1::new();

    println!(
        "BIP32 Root Public Key: {}",
        ExtendedPubKey::from_priv(&secp, &root)
    );

    let legacy = ExtendedPubKey::from_priv(
        &secp,
        &root.derive_priv(
            &secp,
            &account_extended_derivation_path(44, network, account)?,
        )?,
    );
    let nested_segwit = ExtendedPubKey::from_priv(
        &secp,
        &root.derive_priv(
            &secp,
            &account_extended_derivation_path(49, network, account)?,
        )?,
    );
    let native_segwit = ExtendedPubKey::from_priv(
        &secp,
        &root.derive_priv(
            &secp,
            &account_extended_derivation_path(84, network, account)?,
        )?,
    );
    let taproot = ExtendedPubKey::from_priv(
        &secp,
        &root.derive_priv(
            &secp,
            &account_extended_derivation_path(86, network, account)?,
        )?,
    );

    let legacy: Descriptor<String> = descriptor(legacy, network, 44, account, false)?;
    let nested_segwit: Descriptor<String> = descriptor(nested_segwit, network, 49, account, false)?;
    let native_segwit: Descriptor<String> = descriptor(native_segwit, network, 84, account, false)?;
    let taproot: Descriptor<String> = descriptor(taproot, network, 86, account, false)?;

    println!("Legacy: {}", legacy);
    println!("Nested Segwit: {}", nested_segwit);
    println!("Native Segwit: {}", native_segwit);
    println!("Taproot: {}", taproot);

    Ok(())
}

pub fn derive<S, PSW>(
    file_name: S,
    get_password: PSW,
    network: Network,
    word_count: WordCount,
    index: Index,
) -> Result<()>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let root: ExtendedPrivKey = extended_private_key(file_name, get_password, network)?;
    let secp = Secp256k1::new();

    let mnemonic: Mnemonic = Mnemonic::from_bip85(&secp, &root, word_count, index)?;
    println!("Mnemonic: {}", mnemonic);
    Ok(())
}

pub fn sign<S, PSW>(name: S, get_password: PSW, network: Network, psbt_file: PathBuf) -> Result<()>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    if !psbt_file.exists() && !psbt_file.is_file() {
        return Err(anyhow!("PSBT file not found."));
    }

    let mut file: File = File::open(psbt_file.clone())?;
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;

    let psbt: String = base64::encode(content);
    let mut psbt = PartiallySignedTransaction::from_str(&psbt)?;

    let root: ExtendedPrivKey = extended_private_key(name, get_password, network)?;
    let secp = Secp256k1::new();
    let root_fingerprint: Fingerprint = root.fingerprint(&secp);

    let mut paths: Vec<DerivationPath> = Vec::new();

    for input in psbt.inputs.iter() {
        for (fingerprint, path) in input.bip32_derivation.values() {
            if fingerprint.eq(&root_fingerprint) {
                paths.push(path.clone());
            }
        }
    }

    if paths.is_empty() {
        return Err(anyhow!("Nothing to sign here."));
    }

    let mut finalized: bool = false;

    for path in paths.into_iter() {
        let child_priv: ExtendedPrivKey = root.derive_priv(&secp, &path)?;
        let desc = DescriptorSecretKey::from_str(&child_priv.to_string())?;
        let descriptor = match path.into_iter().next() {
            Some(ChildNumber::Hardened { index: 44 }) => descriptor!(pkh(desc))?,
            Some(ChildNumber::Hardened { index: 49 }) => descriptor!(sh(wpkh(desc)))?,
            Some(ChildNumber::Hardened { index: 84 }) => descriptor!(wpkh(desc))?,
            Some(ChildNumber::Hardened { index: 86 }) => descriptor!(tr(desc))?,
            _ => return Err(anyhow!("Unsupported derivation path")),
        };

        let wallet = Wallet::new(descriptor, None, network, MemoryDatabase::default())?;

        // Required for sign
        let _ = wallet.get_address(AddressIndex::New)?;

        if wallet.sign(&mut psbt, SignOptions::default())? {
            finalized = true;
        }
    }

    if finalized {
        let mut psbt_file = psbt_file;
        rename_psbt_to_signed(&mut psbt_file)?;
        let mut file: File = File::options()
            .create_new(true)
            .write(true)
            .open(psbt_file)?;
        file.write_all(psbt.to_string().as_bytes())?;
        println!("Signed.")
    } else {
        println!("PSBT signing not finalized");
    }

    Ok(())
}

fn rename_psbt_to_signed(psbt_file: &mut PathBuf) -> Result<()> {
    if let Some(mut file_name) = psbt_file.file_name().and_then(OsStr::to_str) {
        if let Some(ext) = psbt_file.extension().and_then(OsStr::to_str) {
            let splitted: Vec<&str> = file_name.split(&format!(".{}", ext)).collect();
            file_name = match splitted.first() {
                Some(name) => *name,
                None => return Err(anyhow!("Impossible to get file name")),
            }
        }
        psbt_file.set_file_name(&format!("{}-signed.psbt", file_name));
        Ok(())
    } else {
        Err(anyhow!("Impossible to get file name"))
    }
}

pub fn identity<S, PSW>(name: S, get_password: PSW, network: Network) -> Result<()>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let root: ExtendedPrivKey = extended_private_key(name, get_password, network)?;
    let secp = Secp256k1::new();
    println!("Fingerprint: {}", root.fingerprint(&secp));
    Ok(())
}

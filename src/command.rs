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

use crate::types::{Index, Seed, WordCount};
use crate::util::bip::bip85::FromBip85;
use crate::util::{self, aes, dir};

pub fn restore<S>(file_name: S, password: S, mnemonic: S, passphrase: Option<S>) -> Result<()>
where
    S: Into<String>,
{
    let keychain_file: PathBuf = dir::get_directory()?.join(file_name.into());
    if keychain_file.exists() {
        return Err(anyhow!(
            "There is already a file with the same name! Please, choose another name."
        ));
    }

    let password: String = password.into();
    if password.is_empty() {
        return Err(anyhow!("Invalid password"));
    }

    let seed = Seed::new(mnemonic, passphrase)?;
    let serialized_seed: Vec<u8> = util::serialize(seed)?;
    let encrypted_seed: Vec<u8> = aes::encrypt(password, &serialized_seed);

    let mut file: File = File::options()
        .create_new(true)
        .write(true)
        .open(keychain_file)?;
    file.write_all(&encrypted_seed)?;

    Ok(())
}

pub fn open<S>(file_name: S, password: S) -> Result<Seed>
where
    S: Into<String>,
{
    let keychain_file: PathBuf = dir::get_directory()?.join(file_name.into());

    // Check if mnemonic file exist
    if !keychain_file.exists() {
        return Err(anyhow!("File not found."));
    }

    // Read seed from file
    let mut file: File = File::open(keychain_file)?;
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;

    // Decrypt seed
    match aes::decrypt(password.into(), &content) {
        Ok(data) => util::deserialize(data),
        Err(aes::Error::WrongBlockMode) => Err(anyhow!(
            "Impossible to decrypt file: invalid password or content"
        )),
        Err(e) => Err(anyhow!(e.to_string())),
    }
}

pub fn view_seed<S>(file_name: S, password: S) -> Result<()>
where
    S: Into<String>,
{
    let seed: Seed = open(file_name, password)?;
    println!("\n################################################################\n");
    println!("Mnemonic: {}", seed.mnemonic());
    if let Some(passphrase) = seed.passphrase() {
        println!("Passphrase: {}", passphrase);
    }
    println!("Seed (hex format): {}", seed.to_hex());
    println!("\n################################################################\n");
    Ok(())
}

pub fn wipe<S>(file_name: S, password: S) -> Result<()>
where
    S: Into<String> + Clone,
{
    let _ = open(file_name.clone(), password)?;
    let keychain_file: PathBuf = dir::get_directory()?.join(file_name.into());
    std::fs::remove_file(keychain_file)?;
    Ok(())
}

pub fn extended_private_key<S>(
    file_name: S,
    password: S,
    network: Network,
) -> Result<ExtendedPrivKey>
where
    S: Into<String>,
{
    let seed: Seed = open(file_name, password)?;
    Ok(ExtendedPrivKey::new_master(network, &seed.to_bytes())?)
}

fn account_extended_derivation_path(purpose: u32, account: Option<u32>) -> Result<DerivationPath> {
    // Path: m/<purpose>'/0'/<account>'
    let path: Vec<ChildNumber> = vec![
        ChildNumber::from_hardened_idx(purpose)?,
        ChildNumber::from_hardened_idx(0)?,
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

pub fn get_public_keys<S>(
    file_name: S,
    password: S,
    network: Network,
    account: Option<u32>,
) -> Result<()>
where
    S: Into<String>,
{
    let root: ExtendedPrivKey = extended_private_key(file_name, password, network)?;

    let secp = Secp256k1::new();
    let legacy = ExtendedPubKey::from_priv(
        &secp,
        &root.derive_priv(&secp, &account_extended_derivation_path(44, account)?)?,
    );
    let nested_segwit = ExtendedPubKey::from_priv(
        &secp,
        &root.derive_priv(&secp, &account_extended_derivation_path(49, account)?)?,
    );
    let native_segwit = ExtendedPubKey::from_priv(
        &secp,
        &root.derive_priv(&secp, &account_extended_derivation_path(84, account)?)?,
    );
    let taproot = ExtendedPubKey::from_priv(
        &secp,
        &root.derive_priv(&secp, &account_extended_derivation_path(86, account)?)?,
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

pub fn derive<S>(
    file_name: S,
    password: S,
    network: Network,
    word_count: WordCount,
    index: Index,
) -> Result<()>
where
    S: Into<String>,
{
    let root: ExtendedPrivKey = extended_private_key(file_name, password, network)?;
    let secp = Secp256k1::new();

    let mnemonic: Mnemonic = Mnemonic::from_bip85(&secp, &root, word_count, index)?;
    println!("Mnemonic: {}", mnemonic);
    Ok(())
}

pub fn sign<S>(file_name: S, password: S, network: Network, psbt_file: PathBuf) -> Result<()>
where
    S: Into<String>,
{
    if !psbt_file.exists() && !psbt_file.is_file() {
        return Err(anyhow!("PSBT file not found."));
    }

    let mut file: File = File::open(psbt_file.clone())?;
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;

    let psbt: String = base64::encode(content);
    let mut psbt = PartiallySignedTransaction::from_str(&psbt)?;

    let root: ExtendedPrivKey = extended_private_key(file_name, password, network)?;
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

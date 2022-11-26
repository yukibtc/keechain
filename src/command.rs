// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bdk::miniscript::Descriptor;
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey, ExtendedPubKey};
use bitcoin::Network;
use secp256k1::Secp256k1;

use crate::types::Seed;
use crate::util::{self, aes, dir};

pub struct KeeChain {
    keychain_file: PathBuf,
}

pub fn restore<S>(file_name: S, password: S, mnemonic: S, passphrase: Option<S>) -> Result<()>
where
    S: Into<String>,
{
    let keychain_file: PathBuf = dir::get_directory()?.join(file_name.into());

    // Check if mnemonic file already exist
    if keychain_file.exists() {
        return Err(anyhow!(
            "There is already a file with the same name! Please, choose another name."
        ));
    }

    let password: String = password.into();

    // Check if password is valid
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

pub fn extended_public_key<S>(file_name: S, password: S, network: Network) -> Result<ExtendedPubKey>
where
    S: Into<String>,
{
    let privkey = extended_private_key(file_name, password, network)?;
    let secp = Secp256k1::new();
    Ok(ExtendedPubKey::from_priv(&secp, &privkey))
}

fn derivation_path(purpose: u8, account: Option<u32>) -> Result<DerivationPath> {
    Ok(DerivationPath::from_str(&format!(
        "m/{}'/0'/{}'",
        purpose,
        account.unwrap_or(0),
    ))?)
}

fn descriptor(
    pubkey: ExtendedPubKey,
    purpose: u8,
    account: Option<u32>,
    change: bool,
) -> Result<Descriptor<String>> {
    let descriptor: String = format!(
        "[{}/{}'/0'/{}']{}/{}/*",
        pubkey.fingerprint(),
        purpose,
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
    let xpriv: ExtendedPrivKey = extended_private_key(file_name, password, network)?;

    let secp = Secp256k1::new();
    let legacy = ExtendedPubKey::from_priv(
        &secp,
        &xpriv.derive_priv(&secp, &derivation_path(44, account)?)?,
    );
    let nested_segwit = ExtendedPubKey::from_priv(
        &secp,
        &xpriv.derive_priv(&secp, &derivation_path(49, account)?)?,
    );
    let native_segwit = ExtendedPubKey::from_priv(
        &secp,
        &xpriv.derive_priv(&secp, &derivation_path(84, account)?)?,
    );
    let taproot = ExtendedPubKey::from_priv(
        &secp,
        &xpriv.derive_priv(&secp, &derivation_path(86, account)?)?,
    );

    let legacy: Descriptor<String> = descriptor(legacy, 44, account, false)?;
    let nested_segwit: Descriptor<String> = descriptor(nested_segwit, 49, account, false)?;
    let native_segwit: Descriptor<String> = descriptor(native_segwit, 84, account, false)?;
    let taproot: Descriptor<String> = descriptor(taproot, 86, account, false)?;

    println!("Legacy: {}", legacy);
    println!("Nested Segwit: {}", nested_segwit);
    println!("Native Segwit: {}", native_segwit);
    println!("Taproot: {}", taproot);

    Ok(())
}

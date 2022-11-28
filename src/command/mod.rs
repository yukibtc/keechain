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
use bitcoin::hashes::hmac::{Hmac, HmacEngine};
use bitcoin::hashes::{sha512, Hash, HashEngine};
use bitcoin::psbt::PartiallySignedTransaction;
use bitcoin::util::bip32::{
    ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint,
};
use bitcoin::Network;
use rand::rngs::OsRng;
use rand::RngCore;
use secp256k1::Secp256k1;
use sysinfo::{System, SystemExt};

pub mod danger;
pub mod export;

use crate::types::{Index, Seed, WordCount};
use crate::util::aes::Aes256Encryption;
use crate::util::bip::bip85::FromBip85;
use crate::util::{dir, time};

fn entropy(word_count: WordCount) -> Vec<u8> {
    let mut h = HmacEngine::<sha512::Hash>::new(b"keechain-entropy");

    // OS Random
    let mut os_random = [0u8; 32];
    OsRng.fill_bytes(&mut os_random);
    h.input(&os_random);

    if System::IS_SUPPORTED {
        let system_info: System = System::new_all();

        // Dynamic events
        let dynamic_events: Vec<u8> = vec![
            time::timestamp_nanos().to_be_bytes().to_vec(),
            system_info.boot_time().to_be_bytes().to_vec(),
            system_info.free_memory().to_be_bytes().to_vec(),
            system_info.free_swap().to_be_bytes().to_vec(),
            format!("{:?}", system_info.processes()).as_bytes().to_vec(),
            format!("{:?}", system_info.load_average())
                .as_bytes()
                .to_vec(),
        ]
        .concat();

        h.input(&dynamic_events);

        // Static events
        let static_events: Vec<u8> = vec![
            format!("{:?}", system_info.host_name()).as_bytes().to_vec(),
            format!("{:?}", system_info.long_os_version())
                .as_bytes()
                .to_vec(),
            format!("{:?}", system_info.kernel_version())
                .as_bytes()
                .to_vec(),
            format!("{:?}", system_info.global_cpu_info())
                .as_bytes()
                .to_vec(),
        ]
        .concat();

        h.input(&static_events);
    } else {
        println!("Impossible to fetch entropy from dynamic and static events: using only OS random and timestamp!");
        println!("For a better entropy use another OS or dice roll generation");
        h.input(&time::timestamp_nanos().to_be_bytes());
    }

    let entropy: [u8; 64] = Hmac::from_engine(h).into_inner();
    let len: u32 = word_count.as_u32() * 4 / 3;
    entropy[0..len as usize].to_vec()
}

pub fn generate<S, PSW, P>(
    name: S,
    get_password: PSW,
    get_passphrase: P,
    word_count: WordCount,
) -> Result<Mnemonic>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
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

    let entropy: Vec<u8> = entropy(word_count);
    let mnemonic = Mnemonic::from_entropy(&entropy)?;
    let passphrase: Option<String> = get_passphrase()?;
    let seed = Seed::new(mnemonic.to_string(), passphrase)?;

    let mut file: File = File::options()
        .create_new(true)
        .write(true)
        .open(keychain_file)?;
    file.write_all(&seed.encrypt(password)?)?;

    Ok(mnemonic)
}

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

fn extended_private_key<S, PSW>(
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

fn descriptor(
    root_fingerprint: Fingerprint,
    pubkey: ExtendedPubKey,
    path: &DerivationPath,
    change: bool,
) -> Result<Descriptor<String>> {
    let mut iter_path = path.into_iter();

    let purpose: &ChildNumber = match iter_path.next() {
        Some(child) => child,
        None => return Err(anyhow!("Invalid derivation path: purpose not provided")),
    };

    let coin: &ChildNumber = match iter_path.next() {
        Some(ChildNumber::Hardened { index: 0 }) => &ChildNumber::Hardened { index: 0 },
        Some(ChildNumber::Hardened { index: 1 }) => &ChildNumber::Hardened { index: 1 },
        _ => {
            return Err(anyhow!(
                "Invalid derivation path: coin invalid or not provided"
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
        _ => return Err(anyhow!("Unsupported derivation path")),
    };

    Ok(Descriptor::from_str(&descriptor)?)
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

pub fn decode(psbt_file: PathBuf) -> Result<PartiallySignedTransaction> {
    if !psbt_file.exists() && !psbt_file.is_file() {
        return Err(anyhow!("PSBT file not found."));
    }

    let mut file: File = File::open(psbt_file)?;
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;

    let psbt: String = base64::encode(content);
    Ok(PartiallySignedTransaction::from_str(&psbt)?)
}

pub fn sign<S, PSW>(name: S, get_password: PSW, network: Network, psbt_file: PathBuf) -> Result<()>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let mut psbt: PartiallySignedTransaction = decode(psbt_file.clone())?;

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
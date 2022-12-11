// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use bdk::descriptor;
use bdk::keys::bip39::Mnemonic;
use bdk::miniscript::Descriptor;
use bitcoin::hashes::hmac::{Hmac, HmacEngine};
use bitcoin::hashes::{sha512, Hash, HashEngine};
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPubKey, Fingerprint};
use bitcoin::Network;
use rand::rngs::OsRng;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rand_hc::Hc128Rng;
use sysinfo::{System, SystemExt};

pub mod advanced;
pub mod export;
pub mod psbt;
pub mod setting;

use crate::crypto::aes::Aes256Encryption;
use crate::error::{Error, Result};
use crate::types::{Seed, WordCount};
use crate::util::bip::bip32::Bip32RootKey;
use crate::util::{dir, time};

fn entropy(word_count: WordCount, custom: Option<Vec<u8>>) -> Vec<u8> {
    let mut h = HmacEngine::<sha512::Hash>::new(b"keechain-entropy");

    // TRNG & CSPRNG
    let mut os_random = [0u8; 32];
    OsRng.fill_bytes(&mut os_random);
    h.input(&os_random);

    let mut hc = Hc128Rng::from_entropy();
    let mut hc_random = [0u8; 32];
    hc.fill_bytes(&mut hc_random);
    h.input(&hc_random);

    let mut chacha = ChaCha20Rng::from_entropy();
    let mut chacha_random = [0u8; 32];
    chacha.fill_bytes(&mut chacha_random);
    h.input(&chacha_random);

    if System::IS_SUPPORTED {
        let system_info: System = System::new_all();

        // Dynamic events
        let dynamic_events: Vec<u8> = vec![
            time::timestamp_nanos().to_be_bytes().to_vec(),
            system_info.boot_time().to_be_bytes().to_vec(),
            system_info.total_memory().to_be_bytes().to_vec(),
            system_info.free_memory().to_be_bytes().to_vec(),
            system_info.total_swap().to_be_bytes().to_vec(),
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
            system_info
                .host_name()
                .unwrap_or_else(|| rand::random::<u128>().to_string())
                .as_bytes()
                .to_vec(),
            system_info
                .long_os_version()
                .unwrap_or_else(|| rand::random::<u128>().to_string())
                .as_bytes()
                .to_vec(),
            system_info
                .kernel_version()
                .unwrap_or_else(|| rand::random::<u128>().to_string())
                .as_bytes()
                .to_vec(),
            format!("{:?}", system_info.global_cpu_info())
                .as_bytes()
                .to_vec(),
            format!("{:?}", system_info.users()).as_bytes().to_vec(),
        ]
        .concat();

        h.input(&static_events);
    } else {
        log::warn!("impossible to fetch entropy from dynamic and static events");
        h.input(&time::timestamp_nanos().to_be_bytes());
    }

    // Add custom entropy
    if let Some(custom) = custom {
        h.input(&custom);
    }

    let entropy: [u8; 64] = Hmac::from_engine(h).into_inner();
    let len: u32 = word_count.as_u32() * 4 / 3;
    entropy[0..len as usize].to_vec()
}

pub fn generate<S, PSW, P, E>(
    name: S,
    get_password: PSW,
    get_passphrase: P,
    word_count: WordCount,
    get_custom_entropy: E,
) -> Result<Seed>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
    P: FnOnce() -> Result<Option<String>>,
    E: FnOnce() -> Result<Option<Vec<u8>>>,
{
    let keychain_file: PathBuf = dir::get_keychain_file(name)?;
    if keychain_file.exists() {
        return Err(Error::Generic(
            "There is already a file with the same name! Please, choose another name.".to_string(),
        ));
    }

    let password: String = get_password()?;
    if password.is_empty() {
        return Err(Error::Generic("Invalid password".to_string()));
    }

    let custom_entropy: Option<Vec<u8>> = get_custom_entropy()?;
    let entropy: Vec<u8> = entropy(word_count, custom_entropy);
    let mnemonic = Mnemonic::from_entropy(&entropy)?;
    let passphrase: Option<String> = get_passphrase()?;
    let seed = Seed::new(mnemonic, passphrase);

    let mut file: File = File::options()
        .create_new(true)
        .write(true)
        .open(keychain_file)?;
    file.write_all(&seed.encrypt(password)?)?;

    Ok(seed)
}

pub fn restore<S, PSW, M, P>(
    name: S,
    get_password: PSW,
    get_mnemonic: M,
    get_passphrase: P,
) -> Result<Seed>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
    M: FnOnce() -> Result<Mnemonic>,
    P: FnOnce() -> Result<Option<String>>,
{
    let keychain_file: PathBuf = dir::get_keychain_file(name)?;
    if keychain_file.exists() {
        return Err(Error::Generic(
            "There is already a file with the same name! Please, choose another name.".to_string(),
        ));
    }

    let password: String = get_password()?;
    if password.is_empty() {
        return Err(Error::Generic("Invalid password".to_string()));
    }

    let mnemonic: Mnemonic = get_mnemonic()?;
    let passphrase: Option<String> = get_passphrase()?;
    let seed = Seed::new(mnemonic, passphrase);

    let mut file: File = File::options()
        .create_new(true)
        .write(true)
        .open(keychain_file)?;
    file.write_all(&seed.encrypt(password)?)?;

    Ok(seed)
}

pub fn open<S, PSW>(name: S, get_password: PSW) -> Result<Seed>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let keychain_file: PathBuf = dir::get_keychain_file(name)?;

    // Check if mnemonic file exist
    if !keychain_file.exists() {
        return Err(Error::Generic("File not found.".to_string()));
    }

    // Read seed from file
    let mut file: File = File::open(keychain_file)?;
    let mut content: Vec<u8> = Vec::new();
    file.read_to_end(&mut content)?;

    let password: String = get_password()?;

    Seed::decrypt(password, &content)
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

    Ok(Descriptor::from_str(&descriptor)
        .map_err(|e| Error::Parse(format!("Impossible to parse descriptor: {}", e)))?)
}

pub fn identity<S, PSW>(name: S, get_password: PSW, network: Network) -> Result<Fingerprint>
where
    S: Into<String>,
    PSW: FnOnce() -> Result<String>,
{
    let seed: Seed = open(name, get_password)?;
    seed.fingerprint(network)
}

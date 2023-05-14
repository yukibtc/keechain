// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use bitcoin::Network;
#[cfg(feature = "nostr")]
use nostr::nips::nip06::FromMnemonic;
#[cfg(feature = "nostr")]
use nostr::Keys;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

use super::Descriptors;
use crate::bips::bip32::{self, Bip32, Fingerprint};
use crate::bips::bip39::{self, Mnemonic};
use crate::bips::bip85::{self, Bip85};
use crate::crypto::aes::{self, Aes256Encryption};
use crate::types::{Index, Secrets, Seed, WordCount};
use crate::util::dir::KEECHAIN_EXTENSION;
use crate::util::{self, base64};
use crate::Result;

const KEECHAIN_FILE_VERSION: u8 = 1;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Aes(#[from] aes::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    BIP32(#[from] bip32::Error),
    #[error(transparent)]
    BIP39(#[from] bip39::Error),
    #[error(transparent)]
    BIP85(#[from] bip85::Error),
    #[error(transparent)]
    Descriptors(#[from] super::descriptors::Error),
    #[cfg(feature = "nostr")]
    #[error(transparent)]
    Nostr(#[from] nostr::nips::nip06::Error),
    #[error("File not found")]
    FileNotFound,
    #[error("There is already a file with the same name! Please, choose another name")]
    FileAlreadyExists,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Impossible to decrypt file: invalid password or content")]
    DecryptionFailed,
    #[error("{0}")]
    Generic(String),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum EncryptionKeyType {
    Password,
    // GPG { key_id: String },
}

#[derive(Serialize, Deserialize)]
struct KeeChainRaw {
    version: u8,
    encryption_key_type: EncryptionKeyType,
    keychain: String,
}

#[derive(Debug, Clone)]
pub struct KeeChain {
    file: PathBuf,
    password: String,
    version: u8,
    encryption_key_type: EncryptionKeyType,
    pub keychain: Keychain,
}

impl KeeChain {
    pub fn new<P, S>(
        file: P,
        password: S,
        version: u8,
        encryption_key_type: EncryptionKeyType,
        keychain: Keychain,
    ) -> Self
    where
        P: AsRef<Path>,
        S: Into<String>,
    {
        Self {
            file: file.as_ref().to_path_buf(),
            password: password.into(),
            version,
            encryption_key_type,
            keychain,
        }
    }

    pub fn open<P, PSW>(path: P, get_password: PSW) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        PSW: FnOnce() -> Result<String>,
    {
        let keychain_file: PathBuf = path.as_ref().to_path_buf();
        if !keychain_file.exists() {
            return Err(Error::FileNotFound);
        }

        let mut file: File = File::open(keychain_file.as_path())?;
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content)?;

        let password: String = get_password().map_err(|e| Error::Generic(e.to_string()))?;

        let keechain_raw_file: KeeChainRaw = util::serde::deserialize(content)?;
        let content: Vec<u8> = base64::decode(keechain_raw_file.keychain)?;

        Ok(Self {
            file: keychain_file,
            password: password.clone(),
            version: keechain_raw_file.version,
            encryption_key_type: keechain_raw_file.encryption_key_type,
            keychain: Keychain::decrypt(password, &content)?,
        })
    }

    pub fn generate<P, PSW, E>(
        path: P,
        get_password: PSW,
        word_count: WordCount,
        get_custom_entropy: E,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        PSW: FnOnce() -> Result<String>,
        E: FnOnce() -> Result<Option<Vec<u8>>>,
    {
        let keychain_file: PathBuf = path.as_ref().to_path_buf();
        if keychain_file.exists() {
            return Err(Error::FileAlreadyExists);
        }

        let password: String = get_password().map_err(|e| Error::Generic(e.to_string()))?;
        if password.is_empty() {
            return Err(Error::InvalidPassword);
        }

        let custom_entropy: Option<Vec<u8>> =
            get_custom_entropy().map_err(|e| Error::Generic(e.to_string()))?;
        let entropy: Vec<u8> = bip39::entropy(word_count, custom_entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy)?;

        let keechain = Self {
            file: keychain_file,
            password,
            version: KEECHAIN_FILE_VERSION,
            encryption_key_type: EncryptionKeyType::Password,
            keychain: Keychain::new(mnemonic, Vec::new()),
        };

        keechain.save()?;

        Ok(keechain)
    }

    pub fn restore<P, PSW, M>(path: P, get_password: PSW, get_mnemonic: M) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        PSW: FnOnce() -> Result<String>,
        M: FnOnce() -> Result<Mnemonic>,
    {
        let keychain_file: PathBuf = path.as_ref().to_path_buf();
        if keychain_file.exists() {
            return Err(Error::FileAlreadyExists);
        }

        let password: String = get_password().map_err(|e| Error::Generic(e.to_string()))?;
        if password.is_empty() {
            return Err(Error::InvalidPassword);
        }

        let mnemonic: Mnemonic = get_mnemonic().map_err(|e| Error::Generic(e.to_string()))?;

        let keechain = Self {
            file: keychain_file,
            password,
            version: KEECHAIN_FILE_VERSION,
            encryption_key_type: EncryptionKeyType::Password,
            keychain: Keychain::new(mnemonic, Vec::new()),
        };

        keechain.save()?;

        Ok(keechain)
    }

    pub fn save(&self) -> Result<(), Error> {
        let keychain: Vec<u8> = self.keychain.encrypt(self.password.clone())?;
        let raw = KeeChainRaw {
            version: self.version,
            encryption_key_type: self.encryption_key_type.clone(),
            keychain: base64::encode(keychain),
        };
        let data: Vec<u8> = util::serde::serialize(raw)?;
        let mut file: File = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.file.as_path())?;
        file.write_all(&data)?;
        Ok(())
    }

    pub fn check_password<S>(&self, password: S) -> bool
    where
        S: Into<String>,
    {
        self.password == password.into()
    }

    pub fn rename<S>(&mut self, new_name: S) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let mut new: PathBuf = self.file.clone();
        new.set_file_name(new_name.into());
        new.set_extension(KEECHAIN_EXTENSION);
        if new.exists() {
            Err(Error::FileAlreadyExists)
        } else {
            fs::rename(self.file.as_path(), new.as_path())?;
            self.file = new;
            Ok(())
        }
    }

    pub fn change_password<NPSW>(&mut self, get_new_password: NPSW) -> Result<(), Error>
    where
        NPSW: FnOnce() -> Result<String>,
    {
        let new_password: String = get_new_password().map_err(|e| Error::Generic(e.to_string()))?;
        if self.password != new_password {
            self.password = new_password;
            self.save()?;
        }
        Ok(())
    }

    pub fn wipe(&self) -> Result<(), Error> {
        let mut file: File = File::options()
            .write(true)
            .truncate(true)
            .open(self.file.as_path())?;
        file.write_all(&[0u8; 21])?;
        std::fs::remove_file(self.file.as_path())?;
        Ok(())
    }
}

#[derive(Deserialize)]
struct KeychainIntermediate {
    mnemonic: Mnemonic,
    passphrases: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Keychain {
    mnemonic: Mnemonic,
    passphrases: Vec<String>,
    #[serde(skip_serializing)]
    pub seed: Seed,
}

impl<'de> Deserialize<'de> for Keychain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let intermediate = KeychainIntermediate::deserialize(deserializer)?;
        Ok(Self::new(intermediate.mnemonic, intermediate.passphrases))
    }
}

impl Keychain {
    pub fn new(mnemonic: Mnemonic, passphrases: Vec<String>) -> Self {
        Self {
            mnemonic: mnemonic.clone(),
            passphrases,
            seed: Seed::from_mnemonic(mnemonic),
        }
    }

    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic.clone()
    }

    pub fn passphrases(&self) -> Vec<String> {
        self.passphrases.clone()
    }

    pub fn seed(&self) -> Seed {
        self.seed.clone()
    }

    pub fn identity(&self, network: Network) -> Result<Fingerprint, Error> {
        Ok(self.seed.fingerprint(network)?)
    }

    pub fn deterministic_entropy(
        &self,
        word_count: WordCount,
        index: Index,
    ) -> Result<Mnemonic, Error> {
        Ok(self.seed.derive_bip85_mnemonic(word_count, index)?)
    }

    pub fn descriptors(
        &self,
        network: Network,
        account: Option<u32>,
    ) -> Result<Descriptors, Error> {
        Ok(Descriptors::new(self.seed(), network, account)?)
    }

    pub fn secrets(&self, network: Network) -> Result<Secrets, Error> {
        Ok(Secrets::new(self.seed(), network)?)
    }

    #[cfg(feature = "nostr")]
    pub fn nostr_keys(&self) -> Result<Keys, Error> {
        Ok(Keys::from_mnemonic(
            self.seed.mnemonic().to_string(),
            self.seed.passphrase(),
        )?)
    }

    pub fn add_passphrase<S>(&mut self, passphrase: S)
    where
        S: Into<String>,
    {
        let passphrase: String = passphrase.into();
        if !self.passphrases.contains(&passphrase) {
            self.passphrases.push(passphrase);
        }
    }

    pub fn remove_passphrase<S>(&mut self, passphrase: S)
    where
        S: Into<String>,
    {
        let passphrase = passphrase.into();
        if let Some(index) = self.passphrases.iter().position(|p| p == &passphrase) {
            self.remove_passphrase_by_index(index);
        }
    }

    pub fn remove_passphrase_by_index(&mut self, index: usize) {
        self.passphrases.remove(index);
    }

    pub fn get_passphrase(&self, index: usize) -> Option<String> {
        self.passphrases.get(index).cloned()
    }

    pub fn apply_passphrase<S>(&mut self, passphrase: Option<S>)
    where
        S: Into<String>,
    {
        self.seed = Seed::new(self.mnemonic.clone(), passphrase);
    }
}

impl Aes256Encryption for Keychain {
    type Err = Error;
    fn encrypt<K>(&self, key: K) -> Result<Vec<u8>, Self::Err>
    where
        K: AsRef<[u8]>,
    {
        let serialized: Vec<u8> = util::serde::serialize(self)?;
        Ok(aes::encrypt(key, &serialized))
    }

    fn decrypt<K>(key: K, content: &[u8]) -> Result<Self, Self::Err>
    where
        K: AsRef<[u8]>,
    {
        match aes::decrypt(key, content) {
            Ok(data) => Ok(util::serde::deserialize(data)?),
            Err(aes::Error::WrongBlockMode) => Err(Error::DecryptionFailed),
            Err(e) => Err(Error::Aes(e)),
        }
    }
}

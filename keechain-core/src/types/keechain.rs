// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use super::keychain::{self, Keychain};
use crate::bips::bip39::{self, Mnemonic};
use crate::crypto::aes::Aes256Encryption;
use crate::types::WordCount;
use crate::util::dir::{KEECHAIN_DOT_EXTENSION, KEECHAIN_EXTENSION};
use crate::util::{self, base64};
use crate::Result;

const KEECHAIN_FILE_VERSION: u8 = 1;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    BIP39(#[from] bip39::Error),
    #[error(transparent)]
    Keychain(#[from] keychain::Error),
    #[error("RwLock: {0}")]
    RwLock(String),
    #[error("File not found")]
    FileNotFound,
    #[error("There is already a file with the same name! Please, choose another name")]
    FileAlreadyExists,
    #[error("Invalid password")]
    InvalidPassword,
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

#[derive(Clone)]
pub struct KeeChain {
    file: Arc<RwLock<PathBuf>>,
    password: Arc<RwLock<String>>,
    version: u8,
    encryption_key_type: EncryptionKeyType,
    pub keychain: Keychain,
}

impl fmt::Debug for KeeChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<sensitive>")
    }
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
            file: Arc::new(RwLock::new(file.as_ref().to_path_buf())),
            password: Arc::new(RwLock::new(password.into())),
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
        let content: Vec<u8> = base64::decode(keechain_raw_file.keychain)?; // TODO: remove this and bump keechain version file

        Ok(Self::new(
            keychain_file,
            password.clone(),
            keechain_raw_file.version,
            keechain_raw_file.encryption_key_type,
            Keychain::decrypt(password, &content)?,
        ))
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

        let keechain = Self::new(
            keychain_file,
            password,
            KEECHAIN_FILE_VERSION,
            EncryptionKeyType::Password,
            Keychain::new(mnemonic, Vec::new()),
        );

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

        let keechain = Self::new(
            keychain_file,
            password,
            KEECHAIN_FILE_VERSION,
            EncryptionKeyType::Password,
            Keychain::new(mnemonic, Vec::new()),
        );

        keechain.save()?;

        Ok(keechain)
    }

    pub fn file_path(&self) -> Result<PathBuf, Error> {
        Ok(self
            .file
            .read()
            .map_err(|e| Error::RwLock(e.to_string()))?
            .clone())
    }

    /// Get keechain file name
    pub fn name(&self) -> Option<String> {
        let file = self.file.read().ok()?;
        let file_name = file.file_name()?;
        let file_name = file_name.to_str()?.to_string();
        Some(file_name.replace(KEECHAIN_DOT_EXTENSION, ""))
    }

    pub fn save(&self) -> Result<(), Error> {
        let password = self
            .password
            .read()
            .map_err(|e| Error::RwLock(e.to_string()))?;
        let keychain: String = self.keychain.encrypt(password.clone())?;
        let raw = KeeChainRaw {
            version: self.version,
            encryption_key_type: self.encryption_key_type.clone(),
            keychain: base64::encode(keychain), // TODO: remove this and bump keechain version file
        };
        let data: Vec<u8> = util::serde::serialize(raw)?;
        let file = self.file.read().map_err(|e| Error::RwLock(e.to_string()))?;
        let mut file: File = File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file.as_path())?;
        file.write_all(&data)?;
        Ok(())
    }

    pub fn check_password<S>(&self, password: S) -> Result<bool, Error>
    where
        S: Into<String>,
    {
        Ok(*self
            .password
            .read()
            .map_err(|e| Error::RwLock(e.to_string()))?
            == password.into())
    }

    pub fn rename<S>(&self, new_name: S) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let old: PathBuf = self.file_path()?;
        let mut new: PathBuf = old.clone();
        new.set_file_name(new_name.into());
        new.set_extension(KEECHAIN_EXTENSION);
        if new.exists() {
            Err(Error::FileAlreadyExists)
        } else {
            fs::rename(old.as_path(), new.as_path())?;
            let mut file = self
                .file
                .write()
                .map_err(|e| Error::RwLock(e.to_string()))?;
            *file = new;
            Ok(())
        }
    }

    pub fn change_password<NPSW>(&self, get_new_password: NPSW) -> Result<(), Error>
    where
        NPSW: FnOnce() -> Result<String>,
    {
        let mut password = self
            .password
            .write()
            .map_err(|e| Error::RwLock(e.to_string()))?;
        let new_password: String = get_new_password().map_err(|e| Error::Generic(e.to_string()))?;

        if new_password.is_empty() {
            return Err(Error::InvalidPassword);
        }

        if *password != new_password {
            // Set password
            *password = new_password;

            // Drop the RwLock
            drop(password);

            // Re-save the file
            self.save()?;
        }

        Ok(())
    }

    pub fn wipe(&self) -> Result<(), Error> {
        let file = self.file.read().map_err(|e| Error::RwLock(e.to_string()))?;
        let path = file.as_path();
        let mut file: File = File::options().write(true).truncate(true).open(path)?;
        file.write_all(&[0u8; 21])?;
        std::fs::remove_file(path)?;
        Ok(())
    }
}

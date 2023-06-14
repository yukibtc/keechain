// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::keychain::{self, Keychain};
use crate::bips::bip39::{self, Mnemonic};
use crate::crypto::aes::Aes256Encryption;
use crate::types::WordCount;
use crate::util::dir::KEECHAIN_EXTENSION;
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
        let content: Vec<u8> = base64::decode(keechain_raw_file.keychain)?; // TODO: remove this and bump keechain version file

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
        let keychain: String = self.keychain.encrypt(self.password.clone())?;
        let raw = KeeChainRaw {
            version: self.version,
            encryption_key_type: self.encryption_key_type.clone(),
            keychain: base64::encode(keychain), // TODO: remove this and bump keechain version file
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

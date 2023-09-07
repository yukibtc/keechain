// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use bdk::bitcoin::hashes::Hash;
use serde::{Deserialize, Serialize};

use super::keychain::{self, Keychain};
use crate::bips::bip39::{self, Mnemonic};
use crate::crypto::aes;
use crate::crypto::{self, hash, MultiEncryption};
use crate::types::WordCount;
use crate::util::dir::{self, KEECHAIN_DOT_EXTENSION, KEECHAIN_EXTENSION};
use crate::util::{self, base64};
use crate::Result;

const KEECHAIN_FILE_VERSION: u8 = 2;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Crypto(crypto::Error),
    Aes(aes::Error),
    Dir(dir::Error),
    Json(serde_json::Error),
    Base64(base64::DecodeError),
    BIP39(bip39::Error),
    Keychain(keychain::Error),
    RwLock(String),
    Generic(String),
    InvalidName,
    FileNotFound,
    DecryptionFailed,
    FileAlreadyExists,
    InvalidPassword,
    PasswordNotMatch,
    CurrentPasswordNotMatch,
    UnknownVersion(u8),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "IO: {e}"),
            Self::Crypto(e) => write!(f, "Crypto: {e}"),
            Self::Aes(e) => write!(f, "Aes: {e}"),
            Self::Dir(e) => write!(f, "Dir: {e}"),
            Self::Json(e) => write!(f, "Json: {e}"),
            Self::Base64(e) => write!(f, "Base64: {e}"),
            Self::BIP39(e) => write!(f, "BIP39: {e}"),
            Self::Keychain(e) => write!(f, "Keychain: {e}"),
            Self::RwLock(e) => write!(f, "RwLock: {e}"),
            Self::Generic(e) => write!(f, "Generic: {e}"),
            Self::InvalidName => write!(f, "Invalid name"),
            Self::FileNotFound => write!(f, "File not found"),
            Self::FileAlreadyExists => write!(
                f,
                "There is already a file with the same name! Please, choose another name"
            ),
            Self::DecryptionFailed => {
                write!(f, "Impossible to decrypt file: invalid password or content")
            }
            Self::InvalidPassword => write!(f, "Invalid password"),
            Self::PasswordNotMatch => write!(f, "Password not match"),
            Self::CurrentPasswordNotMatch => write!(f, "Current password not match"),
            Self::UnknownVersion(v) => write!(f, "Unknown keechain file version: {v}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<crypto::Error> for Error {
    fn from(e: crypto::Error) -> Self {
        Self::Crypto(e)
    }
}

impl From<aes::Error> for Error {
    fn from(e: aes::Error) -> Self {
        Self::Aes(e)
    }
}

impl From<dir::Error> for Error {
    fn from(e: dir::Error) -> Self {
        Self::Dir(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Self::Base64(e)
    }
}

impl From<bip39::Error> for Error {
    fn from(e: bip39::Error) -> Self {
        Self::BIP39(e)
    }
}

impl From<keychain::Error> for Error {
    fn from(e: keychain::Error) -> Self {
        Self::Keychain(e)
    }
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

    pub fn open<P, S, PSW>(base_path: P, name: S, get_password: PSW) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: Into<String>,
        PSW: FnOnce() -> Result<String>,
    {
        let name: String = name.into();
        if name.is_empty() {
            return Err(Error::InvalidName);
        }

        let keychain_file: PathBuf = dir::get_keychain_file(base_path, name)?;
        if !keychain_file.exists() {
            return Err(Error::FileNotFound);
        }

        let mut file: File = File::open(keychain_file.as_path())?;
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content)?;

        let password: String = get_password().map_err(|e| Error::Generic(e.to_string()))?;

        let keechain_raw_file: KeeChainRaw = util::serde::deserialize(content)?;
        let keychain_unencrypted: String = keechain_raw_file.keychain;

        // Check keechain file version
        let keychain: Keychain = match keechain_raw_file.version {
            1 => {
                let content: Vec<u8> = base64::decode(keychain_unencrypted)?;
                let key: [u8; 32] = hash::sha256(&password).to_byte_array();
                let data: Vec<u8> =
                    aes::decrypt(key, content).map_err(|_| Error::DecryptionFailed)?;
                util::serde::deserialize(data)?
            }
            2 => Keychain::decrypt(&password, keychain_unencrypted.as_bytes())
                .map_err(|_| Error::DecryptionFailed)?,
            v => return Err(Error::UnknownVersion(v)),
        };

        let keechain = Self::new(
            keychain_file,
            password,
            KEECHAIN_FILE_VERSION,
            keechain_raw_file.encryption_key_type,
            keychain,
        );

        // Migrate
        if keechain_raw_file.version < KEECHAIN_FILE_VERSION {
            keechain.save()?;
        }

        Ok(keechain)
    }

    pub fn generate<P, S, PSW, CPSW, E>(
        base_path: P,
        name: S,
        get_password: PSW,
        get_confirm_password: CPSW,
        word_count: WordCount,
        get_custom_entropy: E,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: Into<String>,
        PSW: FnOnce() -> Result<String>,
        CPSW: FnOnce() -> Result<String>,
        E: FnOnce() -> Result<Option<Vec<u8>>>,
    {
        let name: String = name.into();
        if name.is_empty() {
            return Err(Error::InvalidName);
        }

        let keychain_file: PathBuf = dir::get_keychain_file(base_path, name)?;
        if keychain_file.exists() {
            return Err(Error::FileAlreadyExists);
        }

        let password: String = get_password().map_err(|e| Error::Generic(e.to_string()))?;
        if password.is_empty() {
            return Err(Error::InvalidPassword);
        }

        let confirm_password: String =
            get_confirm_password().map_err(|e| Error::Generic(e.to_string()))?;
        if confirm_password.is_empty() {
            return Err(Error::InvalidPassword);
        }

        if password != confirm_password {
            return Err(Error::PasswordNotMatch);
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

    pub fn restore<P, S, PSW, CPSW, M>(
        base_path: P,
        name: S,
        get_password: PSW,
        get_confirm_password: CPSW,
        get_mnemonic: M,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        PSW: FnOnce() -> Result<String>,
        CPSW: FnOnce() -> Result<String>,
        S: Into<String>,
        M: FnOnce() -> Result<Mnemonic>,
    {
        let name: String = name.into();
        if name.is_empty() {
            return Err(Error::InvalidName);
        }

        let keychain_file: PathBuf = dir::get_keychain_file(base_path, name)?;
        if keychain_file.exists() {
            return Err(Error::FileAlreadyExists);
        }

        let password: String = get_password().map_err(|e| Error::Generic(e.to_string()))?;
        if password.is_empty() {
            return Err(Error::InvalidPassword);
        }

        let confirm_password: String =
            get_confirm_password().map_err(|e| Error::Generic(e.to_string()))?;
        if confirm_password.is_empty() {
            return Err(Error::InvalidPassword);
        }

        if password != confirm_password {
            return Err(Error::PasswordNotMatch);
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
        let raw = KeeChainRaw {
            version: self.version,
            encryption_key_type: self.encryption_key_type.clone(),
            keychain: self.keychain.encrypt(password.clone())?,
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

    pub fn change_password<PSW, NPSW, NCPSW>(
        &self,
        get_old_password: PSW,
        get_new_password: NPSW,
        get_new_confirm_password: NCPSW,
    ) -> Result<(), Error>
    where
        PSW: FnOnce() -> Result<String>,
        NPSW: FnOnce() -> Result<String>,
        NCPSW: FnOnce() -> Result<String>,
    {
        let old_password: String = get_old_password().map_err(|e| Error::Generic(e.to_string()))?;
        let new_password: String = get_new_password().map_err(|e| Error::Generic(e.to_string()))?;
        let new_confirm_password: String =
            get_new_confirm_password().map_err(|e| Error::Generic(e.to_string()))?;

        if !self.check_password(old_password)? {
            return Err(Error::CurrentPasswordNotMatch);
        }

        if new_password.is_empty() {
            return Err(Error::InvalidPassword);
        }

        if new_password != new_confirm_password {
            return Err(Error::PasswordNotMatch);
        }

        let mut password = self
            .password
            .write()
            .map_err(|e| Error::RwLock(e.to_string()))?;

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

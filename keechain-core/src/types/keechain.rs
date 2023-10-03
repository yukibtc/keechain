// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;
use core::ops::{Deref, DerefMut};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use bdk::bitcoin::hashes::sha256::Hash as Sha256Hash;
use bdk::bitcoin::hashes::Hash;
use bdk::bitcoin::psbt::PartiallySignedTransaction;
use bdk::bitcoin::secp256k1::{Secp256k1, Signing};
use bdk::bitcoin::{Network, PrivateKey};
use bdk::miniscript::Descriptor;
use bdk::signer::SignerWrapper;
use serde::{Deserialize, Serialize};

use super::keychain::{self, EncryptedKeychain, Keychain};
use super::Index;
use crate::bips::bip32::{self, Bip32, Fingerprint};
use crate::bips::bip39::{self, Mnemonic};
use crate::crypto::aes;
use crate::crypto::{self, hash, MultiEncryption};
use crate::psbt::{self, PsbtUtility};
use crate::types::WordCount;
use crate::util::dir::{self, KEECHAIN_DOT_EXTENSION, KEECHAIN_EXTENSION};
use crate::util::{self, base64};
use crate::{Result, Seed};

const KEECHAIN_FILE_VERSION: u8 = 2;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Crypto(crypto::Error),
    Aes(aes::Error),
    Dir(dir::Error),
    Json(serde_json::Error),
    Base64(base64::DecodeError),
    BIP32(bip32::Error),
    BIP39(bip39::Error),
    Keychain(keychain::Error),
    Psbt(psbt::Error),
    Generic(String),
    InvalidName,
    FileNotFound,
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
            Self::BIP32(e) => write!(f, "BIP32: {e}"),
            Self::BIP39(e) => write!(f, "BIP39: {e}"),
            Self::Keychain(e) => write!(f, "Keychain: {e}"),
            Self::Psbt(e) => write!(f, "Psbt: {e}"),
            Self::Generic(e) => write!(f, "Generic: {e}"),
            Self::InvalidName => write!(f, "Invalid name"),
            Self::FileNotFound => write!(f, "File not found"),
            Self::FileAlreadyExists => write!(
                f,
                "There is already a file with the same name! Please, choose another name"
            ),
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

impl From<bip32::Error> for Error {
    fn from(e: bip32::Error) -> Self {
        Self::BIP32(e)
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

impl From<psbt::Error> for Error {
    fn from(e: psbt::Error) -> Self {
        Self::Psbt(e)
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
    file: PathBuf,
    password_hash: Sha256Hash,
    version: u8,
    encryption_key_type: EncryptionKeyType,
    encrypted_keychain: EncryptedKeychain,
    network: Network,
}

impl fmt::Debug for KeeChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<sensitive>")
    }
}

impl Deref for KeeChain {
    type Target = EncryptedKeychain;
    fn deref(&self) -> &Self::Target {
        &self.encrypted_keychain
    }
}

impl DerefMut for KeeChain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encrypted_keychain
    }
}

impl KeeChain {
    pub fn new<P, S, C>(
        file: P,
        password: S,
        version: u8,
        encryption_key_type: EncryptionKeyType,
        keychain: Keychain,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: Into<String>,
        C: Signing,
    {
        let password: String = password.into();
        Ok(Self {
            file: file.as_ref().to_path_buf(),
            password_hash: Sha256Hash::hash(password.as_bytes()),
            version,
            encryption_key_type,
            encrypted_keychain: EncryptedKeychain::new(
                keychain.seed.to_bip32_root_pubkey(network, secp)?,
                keychain.encrypt(&password)?,
                network,
            ),
            network,
        })
    }

    pub fn open<P, S, PSW, C>(
        base_path: P,
        name: S,
        get_password: PSW,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: Into<String>,
        PSW: FnOnce() -> Result<String>,
        C: Signing,
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
        let keychain_encrypted: String = keechain_raw_file.keychain;

        // Check keechain file version
        let keychain: Keychain = match keechain_raw_file.version {
            1 => {
                let content: Vec<u8> = base64::decode(keychain_encrypted.as_bytes())?;
                let key: [u8; 32] = hash::sha256(&password).to_byte_array();
                let data: Vec<u8> = aes::decrypt(key, content)?;
                util::serde::deserialize(data)?
            }
            2 => Keychain::decrypt(&password, keychain_encrypted.as_bytes())?,
            v => return Err(Error::UnknownVersion(v)),
        };

        let keechain = Self::new(
            keychain_file,
            &password,
            KEECHAIN_FILE_VERSION,
            keechain_raw_file.encryption_key_type,
            keychain,
            network,
            secp,
        )?;

        // Migrate
        if keechain_raw_file.version < KEECHAIN_FILE_VERSION {
            keechain.save()?;
        }

        Ok(keechain)
    }

    pub fn generate<P, S, PSW, CPSW, E, C>(
        base_path: P,
        name: S,
        get_password: PSW,
        get_confirm_password: CPSW,
        word_count: WordCount,
        get_custom_entropy: E,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        S: Into<String>,
        PSW: FnOnce() -> Result<String>,
        CPSW: FnOnce() -> Result<String>,
        E: FnOnce() -> Result<Option<Vec<u8>>>,
        C: Signing,
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
        let keychain = Keychain::new(mnemonic, Vec::new());

        let keechain = Self::new(
            keychain_file,
            &password,
            KEECHAIN_FILE_VERSION,
            EncryptionKeyType::Password,
            keychain,
            network,
            secp,
        )?;

        keechain.save()?;

        Ok(keechain)
    }

    pub fn restore<P, S, PSW, CPSW, M, C>(
        base_path: P,
        name: S,
        get_password: PSW,
        get_confirm_password: CPSW,
        get_mnemonic: M,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
        PSW: FnOnce() -> Result<String>,
        CPSW: FnOnce() -> Result<String>,
        S: Into<String>,
        M: FnOnce() -> Result<Mnemonic>,
        C: Signing,
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
        let keychain = Keychain::new(mnemonic, Vec::new());

        let keechain = Self::new(
            keychain_file,
            &password,
            KEECHAIN_FILE_VERSION,
            EncryptionKeyType::Password,
            keychain,
            network,
            secp,
        )?;

        keechain.save()?;

        Ok(keechain)
    }

    pub fn file_path(&self) -> PathBuf {
        self.file.clone()
    }

    /// Get keechain file name
    pub fn name(&self) -> Option<String> {
        let file = self.file.as_path();
        let file_name = file.file_name()?;
        let file_name = file_name.to_str()?.to_string();
        Some(file_name.replace(KEECHAIN_DOT_EXTENSION, ""))
    }

    pub fn keychain<T>(&self, password: T) -> Result<Keychain, Error>
    where
        T: AsRef<[u8]>,
    {
        if self.check_password(&password) {
            Ok(self.encrypted_keychain.keychain(password)?)
        } else {
            Err(Error::InvalidPassword)
        }
    }

    pub fn seed<T>(&self, password: T) -> Result<Seed, Error>
    where
        T: AsRef<[u8]>,
    {
        Ok(self.keychain(password)?.seed())
    }

    pub fn identity(&self) -> Fingerprint {
        self.encrypted_keychain.fingerprint()
    }

    /// Passphrase
    pub fn passphrase(&self) -> Option<String> {
        self.encrypted_keychain.passphrase()
    }

    pub fn passphrases<T>(&self, password: T) -> Result<Vec<String>, Error>
    where
        T: AsRef<[u8]>,
    {
        Ok(self.keychain(password)?.passphrases())
    }

    pub fn add_passphrase<T, S>(&mut self, password: T, passphrase: S) -> Result<(), Error>
    where
        T: AsRef<[u8]>,
        S: Into<String>,
    {
        self.encrypted_keychain
            .add_passphrase(password, passphrase)?;
        self.save()?;
        Ok(())
    }

    pub fn remove_passphrase<T, S>(&mut self, password: T, passphrase: S) -> Result<(), Error>
    where
        T: AsRef<[u8]>,
        S: Into<String>,
    {
        self.encrypted_keychain
            .remove_passphrase(password, passphrase)?;
        self.save()?;
        Ok(())
    }

    pub fn apply_passphrase<T, S, C>(
        &mut self,
        password: T,
        passphrase: Option<S>,
        secp: &Secp256k1<C>,
    ) -> Result<(), Error>
    where
        T: AsRef<[u8]>,
        S: Into<String>,
        C: Signing,
    {
        Ok(self
            .encrypted_keychain
            .apply_passphrase(password, passphrase, secp)?)
    }

    pub fn clear_passphrase(&mut self) {
        self.encrypted_keychain.passphrase = None;
        self.encrypted_keychain.current_bip32_root_pubkey =
            self.encrypted_keychain.master_bip32_root_pubkey;
    }

    pub fn deterministic_entropy<T, C>(
        &self,
        password: T,
        word_count: WordCount,
        index: Index,
        secp: &Secp256k1<C>,
    ) -> Result<Mnemonic, Error>
    where
        T: AsRef<[u8]>,
        C: Signing,
    {
        Ok(self
            .keychain(password)?
            .deterministic_entropy(word_count, index, secp)?)
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn save(&self) -> Result<(), Error> {
        let raw = KeeChainRaw {
            version: self.version,
            encryption_key_type: self.encryption_key_type.clone(),
            keychain: self.encrypted_keychain.raw(),
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

    pub fn check_password<T>(&self, password: T) -> bool
    where
        T: AsRef<[u8]>,
    {
        let password: &[u8] = password.as_ref();
        self.password_hash == Sha256Hash::hash(password)
    }

    pub fn sign_psbt<T, C>(
        &self,
        password: T,
        psbt: &mut PartiallySignedTransaction,
        descriptor: Option<Descriptor<String>>,
        custom_signers: Vec<SignerWrapper<PrivateKey>>,
        secp: &Secp256k1<C>,
    ) -> Result<bool, Error>
    where
        T: AsRef<[u8]>,
        C: Signing,
    {
        let seed: Seed = self.seed(password)?;
        Ok(psbt.sign_custom(&seed, descriptor, custom_signers, self.network, secp)?)
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

    pub fn change_password<PSW, NPSW, NCPSW>(
        &mut self,
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

        if !self.check_password(old_password) {
            return Err(Error::CurrentPasswordNotMatch);
        }

        if new_password.is_empty() {
            return Err(Error::InvalidPassword);
        }

        if new_password != new_confirm_password {
            return Err(Error::PasswordNotMatch);
        }

        let new_password_hash = Sha256Hash::hash(new_password.as_bytes());

        if self.password_hash != new_password_hash {
            // Set password
            self.password_hash = new_password_hash;

            // Re-save the file
            self.save()?;
        }

        Ok(())
    }

    pub fn wipe(&self) -> Result<(), Error> {
        let path = self.file.as_path();
        let mut file: File = File::options().write(true).truncate(true).open(path)?;
        file.write_all(&[0u8; 21])?;
        std::fs::remove_file(path)?;
        Ok(())
    }
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use bdk::keys::bip39::Mnemonic;
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

use crate::command::entropy;
use crate::crypto::aes::{self, Aes256Encryption};
use crate::error::{Error, Result};
use crate::types::{Seed, WordCount};
use crate::util::{self, dir};

const KEECHAIN_FILE_VERSION: u8 = 1;

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
    pub fn open<S, PSW>(name: S, get_password: PSW) -> Result<Self>
    where
        S: Into<String>,
        PSW: FnOnce() -> Result<String>,
    {
        let keychain_file: PathBuf = dir::get_keychain_file(name)?;
        if !keychain_file.exists() {
            return Err(Error::Generic("File not found.".to_string()));
        }

        let mut file: File = File::open(keychain_file.as_path())?;
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content)?;

        let password: String = get_password()?;

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

    pub fn generate<S, PSW, E>(
        name: S,
        get_password: PSW,
        word_count: WordCount,
        get_custom_entropy: E,
    ) -> Result<Self>
    where
        S: Into<String>,
        PSW: FnOnce() -> Result<String>,
        E: FnOnce() -> Result<Option<Vec<u8>>>,
    {
        let keychain_file: PathBuf = dir::get_keychain_file(name)?;
        if keychain_file.exists() {
            return Err(Error::Generic(
                "There is already a file with the same name! Please, choose another name."
                    .to_string(),
            ));
        }

        let password: String = get_password()?;
        if password.is_empty() {
            return Err(Error::Generic("Invalid password".to_string()));
        }

        let custom_entropy: Option<Vec<u8>> = get_custom_entropy()?;
        let entropy: Vec<u8> = entropy(word_count, custom_entropy);
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

    pub fn restore<S, PSW, M>(name: S, get_password: PSW, get_mnemonic: M) -> Result<Self>
    where
        S: Into<String>,
        PSW: FnOnce() -> Result<String>,
        M: FnOnce() -> Result<Mnemonic>,
    {
        let keychain_file: PathBuf = dir::get_keychain_file(name)?;
        if keychain_file.exists() {
            return Err(Error::Generic(
                "There is already a file with the same name! Please, choose another name."
                    .to_string(),
            ));
        }

        let password: String = get_password()?;
        if password.is_empty() {
            return Err(Error::Generic("Invalid password".to_string()));
        }

        let mnemonic: Mnemonic = get_mnemonic()?;

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

    pub fn save(&self) -> Result<()> {
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

    pub fn rename<S>(&mut self, new_name: S) -> Result<()>
    where
        S: Into<String>,
    {
        let new = dir::get_keychain_file(new_name)?;
        if new.exists() {
            Err(Error::Generic(
                "There is already a file with the same name! Please, choose another name."
                    .to_string(),
            ))
        } else {
            fs::rename(self.file.as_path(), new.as_path())?;
            self.file = new;
            Ok(())
        }
    }

    pub fn change_password<NPSW>(&mut self, get_new_password: NPSW) -> Result<()>
    where
        NPSW: FnOnce() -> Result<String>,
    {
        let new_password: String = get_new_password()?;
        self.password = new_password;
        self.save()?;
        Ok(())
    }

    pub fn wipe(&self) -> Result<()> {
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

    pub fn add_passphrase<S>(&mut self, passphrase: S)
    where
        S: Into<String>,
    {
        let passphrase: String = passphrase.into();
        if !self.passphrases.contains(&passphrase) {
            self.passphrases.push(passphrase);
        }
    }

    pub fn remove_passphrase(&mut self, index: usize) {
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
            Ok(data) => util::serde::deserialize(data),
            Err(aes::Error::WrongBlockMode) => Err(Error::Generic(
                "Impossible to decrypt file: invalid password or content".to_string(),
            )),
            Err(e) => Err(Error::Aes(e)),
        }
    }
}

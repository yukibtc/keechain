// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;

use bdk::bitcoin::hashes::Hash;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod aes;
pub mod chacha20;
pub mod hash;

use crate::util::{self, base64};

#[derive(Debug)]
pub enum Error {
    Aes(aes::Error),
    ChaCha20Poly1305(chacha20::Error),
    Json(serde_json::Error),
    /// Error while decoding from base64
    Base64Decode,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Aes(e) => write!(f, "Aes: {e}"),
            Self::ChaCha20Poly1305(e) => write!(f, "ChaCha20Poly1305: {e}"),
            Self::Json(e) => write!(f, "Json: {e}"),
            Self::Base64Decode => write!(f, "Error while decoding from base64"),
        }
    }
}

impl From<aes::Error> for Error {
    fn from(e: aes::Error) -> Self {
        Self::Aes(e)
    }
}

impl From<chacha20::Error> for Error {
    fn from(e: chacha20::Error) -> Self {
        Self::ChaCha20Poly1305(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

pub(crate) trait MultiEncryption: Sized + Serialize + DeserializeOwned {
    fn hash_key<K>(key: K) -> [u8; 32]
    where
        K: AsRef<[u8]>,
    {
        hash::sha256(key).to_byte_array()
    }

    fn encrypt<K>(&self, key: K) -> Result<String, Error>
    where
        K: AsRef<[u8]>,
    {
        let serialized: Vec<u8> = util::serde::serialize(self)?;
        let key: [u8; 32] = Self::hash_key(key);
        let first_round = aes::encrypt(key, serialized);
        let second_round: Vec<u8> = chacha20::encrypt(key, first_round)?;
        Ok(base64::encode(second_round))
    }

    fn decrypt<K>(key: K, content: &[u8]) -> Result<Self, Error>
    where
        K: AsRef<[u8]>,
    {
        let key: [u8; 32] = Self::hash_key(key);
        let payload: Vec<u8> = base64::decode(content).map_err(|_| Error::Base64Decode)?;
        let first_round: Vec<u8> = chacha20::decrypt(key, payload)?;
        let second_round: Vec<u8> = aes::decrypt(key, first_round)?;
        Ok(util::serde::deserialize(second_round)?)
    }
}

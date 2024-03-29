// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! AES-256 CBC

use core::fmt;

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use bdk::bitcoin::secp256k1::rand;
use cbc::{Decryptor, Encryptor};

use crate::util::base64;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidContentFormat,
    Base64Decode,
    WrongBlockMode,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContentFormat => write!(f, "invalid content format"),
            Self::Base64Decode => write!(f, "error while decoding from base64"),
            Self::WrongBlockMode => write!(f, "invalid password or block mode"),
        }
    }
}

/// Encrypt with AES-256 CBC
pub fn encrypt<T>(key: [u8; 32], content: T) -> String
where
    T: AsRef<[u8]>,
{
    let iv: [u8; 16] = rand::random();
    let cipher: Aes256CbcEnc = Aes256CbcEnc::new(key.as_slice().into(), &iv.into());
    let result: Vec<u8> = cipher.encrypt_padded_vec_mut::<Pkcs7>(content.as_ref());
    format!("{}?iv={}", base64::encode(result), base64::encode(iv))
}

/// Decrypt with AES-256 CBC
pub fn decrypt<T>(key: [u8; 32], content: T) -> Result<Vec<u8>, Error>
where
    T: AsRef<[u8]>,
{
    let content: String = String::from_utf8_lossy(content.as_ref()).to_string();
    let parsed_content: Vec<&str> = content.split("?iv=").collect();
    if parsed_content.len() != 2 {
        return Err(Error::InvalidContentFormat);
    }

    let content: Vec<u8> = base64::decode(parsed_content[0]).map_err(|_| Error::Base64Decode)?;
    let iv: Vec<u8> = base64::decode(parsed_content[1]).map_err(|_| Error::Base64Decode)?;

    let cipher = Aes256CbcDec::new(key.as_slice().into(), iv.as_slice().into());
    let result = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&content)
        .map_err(|_| Error::WrongBlockMode)?;

    Ok(result)
}

#[deprecated]
pub trait Aes256Encryption: Sized {
    type Err;

    fn encrypt<K>(&self, key: K) -> Result<String, Self::Err>
    where
        K: AsRef<[u8]>;

    fn decrypt<K>(key: K, content: &[u8]) -> Result<Self, Self::Err>
    where
        K: AsRef<[u8]>;
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bdk::bitcoin::hashes::Hash;
    use bip39::Mnemonic;

    use super::*;
    use crate::crypto::hash;
    use crate::types::Seed;
    use crate::util;

    #[test]
    fn test_encryption_decryption() {
        let key: &str = "supersecretpassword";
        let key: [u8; 32] = hash::sha256(key).to_byte_array();
        let text: &[u8] = b"My Text";

        let encrypted_content: String = encrypt(key, text);

        assert_eq!(decrypt(key, encrypted_content).unwrap(), text.to_vec());

        assert_eq!(
            decrypt(key, b"invalidcontentformat").unwrap_err(),
            Error::InvalidContentFormat
        );
        assert_eq!(
            decrypt(key, b"badbase64?iv=encode").unwrap_err(),
            Error::Base64Decode
        );

        // Content encrypted with aes256 using GCM mode
        assert_eq!(
            decrypt(
                key,
                b"nseh0cQPEFID5C0CxYdcPwp091NhRQ==?iv=8PHy8/T19vf4+fr7/P3+/w=="
            )
            .unwrap_err(),
            Error::WrongBlockMode
        );
    }

    #[test]
    fn test_encryption_decryption_seed() {
        let key: &str = "supersecretpassword";
        let key: [u8; 32] = hash::sha256(key).to_byte_array();
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);

        let encrypted_seed: String = encrypt(key, &util::serde::serialize(seed.clone()).unwrap());
        let decrypted_seed: Seed =
            util::serde::deserialize(decrypt(key, encrypted_seed).unwrap()).unwrap();

        assert_eq!(decrypted_seed, seed);
    }
}

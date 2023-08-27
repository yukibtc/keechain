// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use core::fmt;

use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::XChaCha20Poly1305;

use crate::util::base64;

/// Error
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// ChaCha20Poly1305 error
    ChaCha20Poly1305(chacha20poly1305::Error),
    /// Error while decoding from base64
    Base64Decode,
    /// Not found in payload
    NotFound(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChaCha20Poly1305(e) => write!(f, "ChaCha20Poly1305: {e}"),
            Self::Base64Decode => write!(f, "Error while decoding from base64"),
            Self::NotFound(value) => write!(f, "{value} not found in payload"),
        }
    }
}

impl From<chacha20poly1305::Error> for Error {
    fn from(e: chacha20poly1305::Error) -> Self {
        Self::ChaCha20Poly1305(e)
    }
}

/// Encrypt
pub fn encrypt<T>(key: [u8; 32], content: T) -> Result<String, Error>
where
    T: AsRef<[u8]>,
{
    // Compose cipher
    let cipher = XChaCha20Poly1305::new(&key.into());

    // Generate 192-bit nonce
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

    // Encrypt
    let ciphertext: Vec<u8> = cipher.encrypt(&nonce, content.as_ref())?;

    // Compose payload
    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(nonce.as_slice());
    payload.extend(ciphertext);

    // Encode payload to base64
    Ok(base64::encode(payload))
}

/// Decrypt
pub fn decrypt<T>(key: [u8; 32], payload: T) -> Result<Vec<u8>, Error>
where
    T: AsRef<[u8]>,
{
    // Decode base64 payload
    let payload: Vec<u8> = base64::decode(payload).map_err(|_| Error::Base64Decode)?;

    // Get data from payload
    let nonce: &[u8] = payload
        .get(0..24)
        .ok_or_else(|| Error::NotFound(String::from("nonce")))?;
    let ciphertext: &[u8] = payload
        .get(24..)
        .ok_or_else(|| Error::NotFound(String::from("ciphertext")))?;

    // Compose cipher
    let cipher = XChaCha20Poly1305::new(&key.into());

    // Decrypt
    Ok(cipher.decrypt(nonce.into(), ciphertext.as_ref())?)
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

        let encrypted_content: String = encrypt(key, text).unwrap();

        assert_eq!(decrypt(key, encrypted_content).unwrap(), text.to_vec());

        assert_eq!(decrypt(key, b"badbase64").unwrap_err(), Error::Base64Decode);
    }

    #[test]
    fn test_encryption_decryption_seed() {
        let key: &str = "supersecretpassword";
        let key: [u8; 32] = hash::sha256(key).to_byte_array();
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);

        let encrypted_seed: String =
            encrypt(key, &util::serde::serialize(seed.clone()).unwrap()).unwrap();
        let decrypted_seed: Seed =
            util::serde::deserialize(decrypt(key, encrypted_seed).unwrap()).unwrap();

        assert_eq!(decrypted_seed, seed);
    }
}

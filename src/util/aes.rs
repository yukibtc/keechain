// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use aes::Aes256;
use cbc::{Decryptor, Encryptor};

use super::hash;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
pub enum Error {
    #[error("Invalid content format")]
    InvalidContentFormat,
    #[error("Error while decoding from base64")]
    Base64DecodeError,
    #[error("Wrong encryption block mode. The content must be encrypted using CBC mode!")]
    WrongBlockMode,
}

pub fn encrypt<T>(key: T, content: &[u8]) -> Vec<u8>
where
    T: AsRef<[u8]>,
{
    let key: Vec<u8> = hash::sha256(key);

    let iv: [u8; 16] = rand::random();
    let cipher: Aes256CbcEnc = Aes256CbcEnc::new(key.as_slice().into(), &iv.into());
    let result: Vec<u8> = cipher.encrypt_padded_vec_mut::<Pkcs7>(content);
    format!("{}?iv={}", base64::encode(result), base64::encode(iv))
        .as_bytes()
        .to_vec()
}

pub fn decrypt<T>(key: T, content: &[u8]) -> Result<Vec<u8>, Error>
where
    T: AsRef<[u8]>,
{
    let content: String = String::from_utf8_lossy(content).to_string();
    let parsed_content: Vec<&str> = content.split("?iv=").collect();
    if parsed_content.len() != 2 {
        return Err(Error::InvalidContentFormat);
    }

    let key: Vec<u8> = hash::sha256(key);
    let content: Vec<u8> =
        base64::decode(parsed_content[0]).map_err(|_| Error::Base64DecodeError)?;
    let iv: Vec<u8> = base64::decode(parsed_content[1]).map_err(|_| Error::Base64DecodeError)?;

    let cipher = Aes256CbcDec::new(key.as_slice().into(), iv.as_slice().into());
    let result = cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&content)
        .map_err(|_| Error::WrongBlockMode)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key: &str = "supersecretpassword";
        let text: &[u8] = b"My Text";

        let encrypted_content: Vec<u8> = encrypt(key, text);

        assert_eq!(decrypt(key, &encrypted_content).unwrap(), text.to_vec());

        assert_eq!(
            decrypt(key, b"invalidcontentformat").unwrap_err(),
            Error::InvalidContentFormat
        );
        assert_eq!(
            decrypt(key, b"badbase64?iv=encode").unwrap_err(),
            Error::Base64DecodeError
        );

        //Content encrypted with aes256 using GCM mode
        assert_eq!(
            decrypt(
                key,
                b"nseh0cQPEFID5C0CxYdcPwp091NhRQ==?iv=8PHy8/T19vf4+fr7/P3+/w=="
            )
            .unwrap_err(),
            Error::WrongBlockMode
        );
    }
}

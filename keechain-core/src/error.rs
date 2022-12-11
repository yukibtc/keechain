// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;

use crate::crypto::aes;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// I/O error
    IO(String),
    /// AES error
    Aes(aes::Error),
    /// ECDSA error
    Secp256k1(bitcoin::secp256k1::Error),
    /// BIP32 error
    BIP32(bitcoin::util::bip32::Error),
    /// BIP39 error
    BIP39(bdk::keys::bip39::Error),
    /// Base58 error
    Base58(bitcoin::util::base58::Error),
    /// Base64 decode error
    Base64(base64::DecodeError),
    /// BDK error
    BDK(String),
    /// Parse error
    Parse(String),
    /// Generic error
    Generic(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(err) => write!(f, "{}", err),
            Self::Aes(err) => write!(f, "{}", err),
            Self::Secp256k1(err) => write!(f, "{}", err),
            Self::BIP32(err) => write!(f, "{}", err),
            Self::BIP39(err) => write!(f, "{}", err),
            Self::Base58(err) => write!(f, "{}", err),
            Self::Base64(err) => write!(f, "{}", err),
            Self::BDK(err) => write!(f, "{}", err),
            Self::Parse(err) => write!(f, "{}", err),
            Self::Generic(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

impl From<aes::Error> for Error {
    fn from(err: aes::Error) -> Self {
        Self::Aes(err)
    }
}

impl From<bitcoin::secp256k1::Error> for Error {
    fn from(err: bitcoin::secp256k1::Error) -> Self {
        Self::Secp256k1(err)
    }
}

impl From<bitcoin::util::bip32::Error> for Error {
    fn from(err: bitcoin::util::bip32::Error) -> Self {
        Self::BIP32(err)
    }
}

impl From<bdk::keys::bip39::Error> for Error {
    fn from(err: bdk::keys::bip39::Error) -> Self {
        Self::BIP39(err)
    }
}

impl From<bitcoin::util::base58::Error> for Error {
    fn from(err: bitcoin::util::base58::Error) -> Self {
        Self::Base58(err)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Self {
        Self::Base64(err)
    }
}

impl From<bdk::Error> for Error {
    fn from(err: bdk::Error) -> Self {
        Self::BDK(err.to_string())
    }
}

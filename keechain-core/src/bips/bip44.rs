// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! BIP44
//!
//! <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>

use core::fmt;

use super::bip32::{self, ChildNumber, DerivationPath};
use super::bip43::Purpose;
use super::bip48::ScriptType;

#[derive(Debug, PartialEq, Eq)]
pub enum UnsupportedDerivationPathError {
    Coin(Option<ChildNumber>),
    Account(Option<ChildNumber>),
    Change(Option<ChildNumber>),
    Purpose(Option<ChildNumber>),
}

impl fmt::Display for UnsupportedDerivationPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Coin(c) => match c {
                Some(c) => write!(f, "`{c}` coin is not supported"),
                None => write!(f, "unknown coin"),
            },
            Self::Account(a) => match a {
                Some(a) => write!(f, "`{a}` account index is not supported"),
                None => write!(f, "unknown account index"),
            },
            Self::Change(c) => match c {
                Some(c) => write!(f, "`{c}` change is not supported"),
                None => write!(f, "unknown change"),
            },
            Self::Purpose(purpose) => match purpose {
                Some(p) => write!(f, "`{p}` purpose is not supported"),
                None => write!(f, "unknown purpose"),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    BIP32(bip32::Error),
    UnsupportedDerivationPath(UnsupportedDerivationPathError),
    BIP48ScriptNotFound,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BIP32(e) => write!(f, "BIP32: {e}"),
            Self::UnsupportedDerivationPath(e) => write!(f, "Unsupported derivation path: {e}"),
            Self::BIP48ScriptNotFound => write!(f, "BIP48 script type not found"),
        }
    }
}

impl From<bip32::Error> for Error {
    fn from(e: bip32::Error) -> Self {
        Self::BIP32(e)
    }
}

pub struct ExtendedPath {
    pub purpose: Purpose,
    pub coin: u32,
    pub account: u32,
    pub change: bool,
}

impl ExtendedPath {
    pub fn from_derivation_path(path: &DerivationPath) -> Result<Self, Error> {
        let mut path = path.into_iter();
        let purpose = path.next();
        let coin = path.next();
        let account = path.next();
        let change = path.next();

        let coin: u32 = match coin {
            Some(ChildNumber::Hardened { index: 0 }) => 0,
            Some(ChildNumber::Hardened { index: 1 }) => 1,
            c => {
                return Err(Error::UnsupportedDerivationPath(
                    UnsupportedDerivationPathError::Coin(c.copied()),
                ))
            }
        };

        let account: u32 = match account {
            Some(ChildNumber::Hardened { index }) => *index,
            a => {
                return Err(Error::UnsupportedDerivationPath(
                    UnsupportedDerivationPathError::Account(a.copied()),
                ))
            }
        };

        let change: bool = match change {
            Some(ChildNumber::Normal { index }) => *index != 0,
            c => {
                return Err(Error::UnsupportedDerivationPath(
                    UnsupportedDerivationPathError::Change(c.copied()),
                ))
            }
        };

        match purpose {
            Some(ChildNumber::Hardened { index: 44 }) => Ok(Self {
                purpose: Purpose::BIP44,
                coin,
                account,
                change,
            }),
            Some(ChildNumber::Hardened { index: 48 }) => {
                let script: ScriptType = match path.next() {
                    Some(ChildNumber::Hardened { index: 1 }) => ScriptType::P2SHWSH,
                    Some(ChildNumber::Hardened { index: 2 }) => ScriptType::P2WSH,
                    Some(ChildNumber::Hardened { index: 3 }) => ScriptType::P2TR,
                    _ => return Err(Error::BIP48ScriptNotFound),
                };
                Ok(Self {
                    purpose: Purpose::BIP48 { script },
                    coin,
                    account,
                    change,
                })
            }
            Some(ChildNumber::Hardened { index: 49 }) => Ok(Self {
                purpose: Purpose::BIP49,
                coin,
                account,
                change,
            }),
            Some(ChildNumber::Hardened { index: 84 }) => Ok(Self {
                purpose: Purpose::BIP84,
                coin,
                account,
                change,
            }),
            Some(ChildNumber::Hardened { index: 86 }) => Ok(Self {
                purpose: Purpose::BIP86,
                coin,
                account,
                change,
            }),
            p => Err(Error::UnsupportedDerivationPath(
                UnsupportedDerivationPathError::Purpose(p.copied()),
            )),
        }
    }
}

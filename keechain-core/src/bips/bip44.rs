// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! BIP44
//!
//! <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>

use core::fmt;
use core::slice::Iter;

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

        let purpose: Option<&ChildNumber> = path.next();

        let coin: u32 = extract_coin(&mut path)?;
        let account: u32 = extract_account(&mut path)?;

        match purpose {
            Some(ChildNumber::Hardened { index: 44 }) => Ok(Self {
                purpose: Purpose::BIP44,
                coin,
                account,
                change: extract_change(&mut path)?,
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
                    change: extract_change(&mut path)?,
                })
            }
            Some(ChildNumber::Hardened { index: 49 }) => Ok(Self {
                purpose: Purpose::BIP49,
                coin,
                account,
                change: extract_change(&mut path)?,
            }),
            Some(ChildNumber::Hardened { index: 84 }) => Ok(Self {
                purpose: Purpose::BIP84,
                coin,
                account,
                change: extract_change(&mut path)?,
            }),
            Some(ChildNumber::Hardened { index: 86 }) => Ok(Self {
                purpose: Purpose::BIP86,
                coin,
                account,
                change: extract_change(&mut path)?,
            }),
            p => Err(Error::UnsupportedDerivationPath(
                UnsupportedDerivationPathError::Purpose(p.copied()),
            )),
        }
    }
}

fn extract_coin(path: &mut Iter<'_, ChildNumber>) -> Result<u32, Error> {
    match path.next() {
        Some(ChildNumber::Hardened { index: 0 }) => Ok(0),
        Some(ChildNumber::Hardened { index: 1 }) => Ok(1),
        c => Err(Error::UnsupportedDerivationPath(
            UnsupportedDerivationPathError::Coin(c.copied()),
        )),
    }
}

fn extract_account(path: &mut Iter<'_, ChildNumber>) -> Result<u32, Error> {
    match path.next() {
        Some(ChildNumber::Hardened { index }) => Ok(*index),
        a => Err(Error::UnsupportedDerivationPath(
            UnsupportedDerivationPathError::Account(a.copied()),
        )),
    }
}

fn extract_change(path: &mut Iter<'_, ChildNumber>) -> Result<bool, Error> {
    match path.next() {
        Some(ChildNumber::Normal { index: 0 }) => Ok(false),
        Some(ChildNumber::Normal { index: 1 }) => Ok(true),
        c => Err(Error::UnsupportedDerivationPath(
            UnsupportedDerivationPathError::Change(c.copied()),
        )),
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;

    #[test]
    fn test_extended_path_parsing() {
        // BIP44
        // BIP48 path
        let path = DerivationPath::from_str("m/86'/0'/22'/1").unwrap();
        let p = ExtendedPath::from_derivation_path(&path).unwrap();
        assert_eq!(p.purpose, Purpose::BIP86);
        assert_eq!(p.coin, 0);
        assert_eq!(p.account, 22);
        assert!(p.change);

        // BIP48 path
        let path = DerivationPath::from_str("m/48'/1'/0'/3'/0").unwrap();
        let p = ExtendedPath::from_derivation_path(&path).unwrap();
        assert_eq!(
            p.purpose,
            Purpose::BIP48 {
                script: ScriptType::P2TR
            }
        );
        assert_eq!(p.coin, 1);
        assert_eq!(p.account, 0);
        assert!(!p.change);
    }
}

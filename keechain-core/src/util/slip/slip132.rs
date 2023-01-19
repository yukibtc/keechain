// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin::util::base58;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPubKey};

use crate::util::hex;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Base58(#[from] bitcoin::util::base58::Error),
    #[error(transparent)]
    Hex(#[from] hex::Error),
    #[error("Unsupported derivation path")]
    UnsupportedDerivationPath,
}

pub trait ToSlip132 {
    type Err;
    fn to_slip132(&self, path: &DerivationPath) -> Result<String, Self::Err>;
}

impl ToSlip132 for ExtendedPubKey {
    type Err = Error;
    fn to_slip132(&self, path: &DerivationPath) -> Result<String, Self::Err> {
        let data: Vec<u8> = base58::from_check(&self.to_string())?;

        let mut iter = path.into_iter();
        let purpose: Option<&ChildNumber> = iter.next();
        let is_mainnet: bool = match iter.next() {
            Some(ChildNumber::Hardened { index: 0 }) => true,
            Some(ChildNumber::Hardened { index: 1 }) => false,
            _ => return Err(Error::UnsupportedDerivationPath),
        };

        let hex: &str = match purpose {
            Some(ChildNumber::Hardened { index: 44 }) => {
                if is_mainnet {
                    "0488b21e"
                } else {
                    "043587cf"
                }
            }
            Some(ChildNumber::Hardened { index: 49 }) => {
                if is_mainnet {
                    "049d7cb2"
                } else {
                    "044a5262"
                }
            }
            Some(ChildNumber::Hardened { index: 84 }) => {
                if is_mainnet {
                    "04b24746"
                } else {
                    "045f1cf6"
                }
            }
            _ => return Err(Error::UnsupportedDerivationPath),
        };

        let data: Vec<u8> = [hex::decode(hex)?, data[4..].to_vec()].concat();
        Ok(base58::check_encode_slice(&data))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bdk::keys::bip39::Mnemonic;
    use bitcoin::secp256k1::Secp256k1;
    use bitcoin::util::bip32::ExtendedPrivKey;
    use bitcoin::Network;

    use super::*;
    use crate::types::Seed;

    #[test]
    fn test_slip132() {
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);

        let root = ExtendedPrivKey::new_master(Network::Bitcoin, &seed.to_bytes()).unwrap();
        let secp = Secp256k1::new();

        let path = DerivationPath::from_str("m/44'/0'/0'").unwrap();
        let pubkey: ExtendedPubKey =
            ExtendedPubKey::from_priv(&secp, &root.derive_priv(&secp, &path).unwrap());
        assert_eq!(pubkey.to_slip132(&path).unwrap(), "xpub6DScrJ7HSQK8XudnGBmuW7Ln9vGfCKYSFP1kyX3UoVo2oj1shjsTj2a3U62ERFnX9rEECxB2EdY8UfarEGLCezmHMTArJtGwRhZxbNkzKwF".to_string());

        let path = DerivationPath::from_str("m/49'/0'/0'").unwrap();
        let pubkey: ExtendedPubKey =
            ExtendedPubKey::from_priv(&secp, &root.derive_priv(&secp, &path).unwrap());
        assert_eq!(pubkey.to_slip132(&path).unwrap(), "ypub6XdTaSG128psDt3wtUyHPRexBo2HYnjDt9JfpiZWhbfV3vBjHfdkot32QkdbnYmBBNxHqG3HW49efmhQGLv3Waudourm6NqDtK4dLdyA3u4".to_string());

        let path = DerivationPath::from_str("m/84'/0'/0'").unwrap();
        let pubkey: ExtendedPubKey =
            ExtendedPubKey::from_priv(&secp, &root.derive_priv(&secp, &path).unwrap());
        assert_eq!(pubkey.to_slip132(&path).unwrap(), "zpub6qR4RRKqYzgY9psfVvZFQchEZfH6upEMWJRJSLWAXeYk4KXNKoLuBzC7977uUKMFiVYNMqMrrjNgJ871YQeJEbgzQ6hZevYE8uB6NipiLLj".to_string());

        assert_eq!(
            pubkey
                .to_slip132(&DerivationPath::from_str("m/1'/0'/0'").unwrap())
                .unwrap_err(),
            Error::UnsupportedDerivationPath
        );
    }
}

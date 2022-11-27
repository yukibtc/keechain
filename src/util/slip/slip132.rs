// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::{anyhow, Result};
use bitcoin::util::base58;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPubKey};

use crate::util::convert;

pub trait ToSlip132 {
    type Err;
    fn to_slip132(&self, path: &DerivationPath) -> Result<String, Self::Err>;
}

impl ToSlip132 for ExtendedPubKey {
    type Err = anyhow::Error;
    fn to_slip132(&self, path: &DerivationPath) -> Result<String, Self::Err> {
        let data = base58::from_check(&self.to_string())?;

        let mut iter = path.into_iter();
        let purpose = iter.next();
        let is_mainnet: bool = match iter.next() {
            Some(coin) => match coin {
                ChildNumber::Hardened { index: 0 } => true,
                ChildNumber::Hardened { index: 1 } => false,
                _ => return Err(anyhow!("Unsupported derivation path")),
            },
            None => return Err(anyhow!("Unsupported derivation path")),
        };

        let hex = match purpose {
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
            _ => return Err(anyhow!("Unsupported derivation path")),
        };

        let data = [convert::hex_to_bytes(hex), data[4..].to_vec()].concat();
        Ok(base58::check_encode_slice(&data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    use bitcoin::util::bip32::ExtendedPrivKey;
    use bitcoin::Network;
    use secp256k1::Secp256k1;

    use crate::types::Seed;

    #[test]
    fn test_slip132() {
        let mnemonic: &str = "easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt";
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase).unwrap();

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

        assert!(pubkey
            .to_slip132(&DerivationPath::from_str("m/1'/0'/0'").unwrap())
            .is_err());
    }
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Instant;

use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey};
use bitcoin::{Address, Network, PublicKey};

use crate::error::{Error, Result};
use crate::types::{Seed, MAX_INDEX};
use crate::util::bip::bip32::{self, Bip32RootKey};

const BECH32_CHARS: &str = "023456789acdefghjklmnpqrstuvwxyz";

pub fn search_address(
    seed: Seed,
    prefixes: impl Into<Vec<String>>,
    network: Network,
) -> Result<(DerivationPath, Address)> {
    let now = Instant::now();
    let secp = Secp256k1::new();
    let root = seed.to_bip32_root_key(network)?;
    let prefixes = prefixes.into();

    for prefix in prefixes.iter() {
        for c in prefix.chars() {
            if !BECH32_CHARS.contains(c) {
                return Err(Error::Generic(format!("Unsupported char: {}", c)));
            }
        }
    }

    for account in 0..MAX_INDEX {
        let path = bip32::account_extended_path(84, network, Some(account))?;
        let derived_private_key: ExtendedPrivKey = root.derive_priv(&secp, &path)?;
        let derived_public_key: ExtendedPubKey =
            ExtendedPubKey::from_priv(&secp, &derived_private_key);

        for index in 0..MAX_INDEX {
            let derived_public_key: ExtendedPubKey =
                derived_public_key.ckd_pub(&secp, ChildNumber::from_normal_idx(index)?)?;

            for change in 0..=1 {
                let derived_public_key: ExtendedPubKey =
                    derived_public_key.ckd_pub(&secp, ChildNumber::from_normal_idx(change)?)?;
                let pubkey = PublicKey::new(derived_public_key.public_key);
                let address = Address::p2wpkh(&pubkey, network).unwrap();
                let addr_str = address.to_string();
                if prefixes.iter().any(|prefix| addr_str.contains(prefix)) {
                    println!("{} ms", now.elapsed().as_millis());
                    let path =
                        bip32::get_path(84, network, Some(account), Some(index), change == 1)?;
                    return Ok((path, address));
                }
            }
        }
    }

    Err(Error::Generic(
        "Failed to derive xpub at found path".to_string(),
    ))
}

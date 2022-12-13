// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin::secp256k1::schnorr::Signature;
use bitcoin::secp256k1::{KeyPair, Message, Secp256k1, SecretKey, XOnlyPublicKey};

use crate::crypto::hash;
use crate::error::Result;

pub fn sign_delegation(
    secret_key: &SecretKey,
    delegatee_pk: XOnlyPublicKey,
    conditions: String,
) -> Result<Signature> {
    let secp = Secp256k1::new();
    let key_pair = KeyPair::from_secret_key(&secp, secret_key);
    let unhashed_token: String = format!("nostr:delegation:{}:{}", delegatee_pk, conditions);
    let hashed_token: Vec<u8> = hash::sha256(unhashed_token.as_bytes());
    let message = Message::from_slice(&hashed_token)?;
    Ok(secp.sign_schnorr(&message, &key_pair))
}

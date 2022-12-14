// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use bech32::{self, ToBase32, Variant};
use bitcoin::secp256k1::{PublicKey, SecretKey};

pub mod nip06;
pub mod nip26;

use crate::error::{Error, Result};

const PREFIX_BECH32_SECRET_KEY: &str = "nsec";
const PREFIX_BECH32_PUBLIC_KEY: &str = "npub";

pub trait ToBech32 {
    type Err;
    fn to_bech32(&self) -> Result<String, Self::Err>;
}

impl ToBech32 for PublicKey {
    type Err = Error;
    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.serialize().to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_PUBLIC_KEY,
            data,
            Variant::Bech32,
        )?)
    }
}

impl ToBech32 for SecretKey {
    type Err = Error;
    fn to_bech32(&self) -> Result<String, Self::Err> {
        let data = self.secret_bytes().to_base32();
        Ok(bech32::encode(
            PREFIX_BECH32_SECRET_KEY,
            data,
            Variant::Bech32,
        )?)
    }
}

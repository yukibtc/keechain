// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![doc = include_str!("../README.md")]

pub extern crate bdk;
pub extern crate bip39;
pub extern crate bitcoin;
#[cfg(feature = "nostr")]
pub extern crate nostr;

use bitcoin::secp256k1::{rand, All, Secp256k1};
use once_cell::sync::Lazy;

pub mod crypto;
pub mod types;
pub mod util;

pub use self::types::{KeeChain, Keychain};

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

pub static SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(|| {
    let mut ctx = Secp256k1::new();
    let mut rng = rand::thread_rng();
    ctx.randomize(&mut rng);
    ctx
});

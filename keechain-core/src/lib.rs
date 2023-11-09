// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

#![allow(clippy::too_many_arguments)]
#![doc = include_str!("../README.md")]

pub extern crate bdk;
pub use bdk::bitcoin;
pub use bdk::bitcoin::hashes;
pub use bdk::bitcoin::secp256k1;
pub use bdk::miniscript;

pub mod bips;
pub mod crypto;
pub mod descriptors;
pub mod export;
pub mod psbt;
pub mod slips;
pub mod types;
pub mod util;

pub use self::descriptors::Descriptors;
pub use self::export::{
    BitcoinCore, ColdcardGenericJson, Electrum, ElectrumSupportedScripts, Wasabi,
};
pub use self::psbt::PsbtUtility;
pub use self::types::{EncryptedKeychain, Index, KeeChain, Keychain, Secrets, Seed, WordCount};

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

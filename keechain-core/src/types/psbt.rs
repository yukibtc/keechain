// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;

use bdk::database::MemoryDatabase;
use bdk::miniscript::descriptor::{DescriptorKeyParseError, DescriptorSecretKey};
use bdk::wallet::AddressIndex;
use bdk::{SignOptions, Wallet};
use bitcoin::psbt::{PartiallySignedTransaction, PsbtParseError};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, Fingerprint};
use bitcoin::Network;

use crate::types::Seed;
use crate::util::base64;
use crate::util::bip::bip32::Bip32RootKey;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    BIP32(#[from] bitcoin::util::bip32::Error),
    #[error(transparent)]
    PsbtParse(#[from] PsbtParseError),
    #[error(transparent)]
    DescriptorParse(#[from] DescriptorKeyParseError),
    #[error(transparent)]
    BDK(#[from] bdk::Error),
    #[error("File not found")]
    FileNotFound,
    #[error("Unsupported derivation path")]
    UnsupportedDerivationPath,
    #[error("Nothing to sign here")]
    NothingToSign,
}

#[derive(Debug, Clone)]
pub struct Psbt {
    psbt: PartiallySignedTransaction,
    network: Network,
}

impl Psbt {
    pub fn new(psbt: PartiallySignedTransaction, network: Network) -> Self {
        Self { psbt, network }
    }

    pub fn from_base64<S>(psbt: S, network: Network) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(Psbt::new(
            PartiallySignedTransaction::from_str(&psbt.into())?,
            network,
        ))
    }

    pub fn from_file<P>(path: P, network: Network) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let psbt_file = path.as_ref();
        if !psbt_file.exists() && !psbt_file.is_file() {
            return Err(Error::FileNotFound);
        }
        let mut file: File = File::open(psbt_file)?;
        let mut content: Vec<u8> = Vec::new();
        file.read_to_end(&mut content)?;
        Self::from_base64(base64::encode(content), network)
    }

    pub fn psbt(&self) -> PartiallySignedTransaction {
        self.psbt.clone()
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn sign(&mut self, seed: &Seed) -> Result<bool, Error> {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(self.network)?;
        let secp = Secp256k1::new();
        let root_fingerprint: Fingerprint = root.fingerprint(&secp);

        let mut paths: Vec<DerivationPath> = Vec::new();

        for input in self.psbt.inputs.iter() {
            for (fingerprint, path) in input.bip32_derivation.values() {
                if fingerprint.eq(&root_fingerprint) {
                    paths.push(path.clone());
                }
            }

            for (_, (fingerprint, path)) in input.tap_key_origins.values() {
                if fingerprint.eq(&root_fingerprint) {
                    paths.push(path.clone());
                }
            }
        }

        if paths.is_empty() {
            return Err(Error::NothingToSign);
        }

        let mut finalized: bool = false;

        for path in paths.into_iter() {
            let child_priv: ExtendedPrivKey = root.derive_priv(&secp, &path)?;
            let desc = DescriptorSecretKey::from_str(&child_priv.to_string())?;
            let descriptor = match path.into_iter().next() {
                Some(ChildNumber::Hardened { index: 44 }) => format!("pkh({desc})"),
                Some(ChildNumber::Hardened { index: 49 }) => format!("sh(wpkh({desc}))"),
                Some(ChildNumber::Hardened { index: 84 }) => format!("wpkh({desc})"),
                Some(ChildNumber::Hardened { index: 86 }) => format!("tr({desc})"),
                _ => return Err(Error::UnsupportedDerivationPath),
            };

            let wallet = Wallet::new(&descriptor, None, self.network, MemoryDatabase::default())?;

            // Required for sign
            let _ = wallet.get_address(AddressIndex::New)?;

            if wallet.sign(&mut self.psbt, SignOptions::default())? {
                finalized = true;
            }
        }
        Ok(finalized)
    }

    pub fn save_to_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let mut file: File = File::options()
            .create_new(true)
            .write(true)
            .open(path.as_ref())?;
        file.write_all(&self.as_bytes()?)?;
        Ok(())
    }

    pub fn as_base64(&self) -> String {
        self.psbt.to_string()
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        Ok(base64::decode(self.as_base64())?)
    }
}

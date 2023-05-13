// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! PSBT

use core::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use bdk::database::MemoryDatabase;
use bdk::miniscript::descriptor::DescriptorKeyParseError;
use bdk::miniscript::Descriptor;
use bdk::signer::{SignerContext, SignerOrdering, SignerWrapper};
use bdk::{KeychainKind, SignOptions, Wallet};
use bitcoin::psbt::{PartiallySignedTransaction, PsbtParseError};
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, Fingerprint};
use bitcoin::{Network, PrivateKey};

use super::descriptors;
use crate::bips::bip32::Bip32RootKey;
use crate::types::{Descriptors, Purpose, Seed};
use crate::util::base64;
use crate::SECP256K1;

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
    Descriptor(#[from] descriptors::Error),
    #[error(transparent)]
    BDK(#[from] bdk::Error),
    #[error("File not found")]
    FileNotFound,
    #[error("Unsupported derivation path")]
    UnsupportedDerivationPath,
    #[error("Invalid derivation path")]
    InvalidDerivationPath,
    #[error("Nothing to sign here")]
    NothingToSign,
    #[error("PSBT not signed")]
    PsbtNotSigned,
}

pub trait Psbt: Sized {
    fn from_base64<S>(psbt: S) -> Result<Self, Error>
    where
        S: Into<String>;

    fn from_file<P>(path: P) -> Result<Self, Error>
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
        Self::from_base64(base64::encode(content))
    }

    fn sign(&mut self, seed: &Seed, network: Network) -> Result<bool, Error> {
        self.sign_custom(seed, None, Vec::new(), network)
    }

    fn sign_with_descriptor(
        &mut self,
        seed: &Seed,
        descriptor: Descriptor<String>,
        network: Network,
    ) -> Result<bool, Error> {
        self.sign_custom(seed, Some(descriptor), Vec::new(), network)
    }

    fn sign_custom(
        &mut self,
        seed: &Seed,
        descriptor: Option<Descriptor<String>>,
        custom_signers: Vec<SignerWrapper<PrivateKey>>,
        network: Network,
    ) -> Result<bool, Error>;

    fn save_to_file<P>(&self, path: P) -> Result<(), Error>
    where
        P: AsRef<Path>,
    {
        let mut file: File = File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path.as_ref())?;
        file.write_all(&self.as_bytes()?)?;
        Ok(())
    }

    fn as_base64(&self) -> String;

    fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        Ok(base64::decode(self.as_base64())?)
    }
}

impl Psbt for PartiallySignedTransaction {
    fn from_base64<S>(psbt: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(PartiallySignedTransaction::from_str(&psbt.into())?)
    }

    fn sign_custom(
        &mut self,
        seed: &Seed,
        descriptor: Option<Descriptor<String>>,
        custom_signers: Vec<SignerWrapper<PrivateKey>>,
        network: Network,
    ) -> Result<bool, Error> {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let root_fingerprint: Fingerprint = root.fingerprint(&SECP256K1);

        let mut paths: Vec<DerivationPath> = Vec::new();

        for input in self.inputs.iter() {
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

        if paths.is_empty() && custom_signers.is_empty() {
            return Err(Error::NothingToSign);
        }

        let descriptor: String = match descriptor {
            Some(desc) => desc.to_string(),
            None => {
                let mut first_path = paths.first().ok_or(Error::NothingToSign)?.into_iter();
                let purpose: Purpose = match first_path.next() {
                    Some(ChildNumber::Hardened { index: 44 }) => Purpose::PKH,
                    Some(ChildNumber::Hardened { index: 49 }) => Purpose::SHWPKH,
                    Some(ChildNumber::Hardened { index: 84 }) => Purpose::WPKH,
                    Some(ChildNumber::Hardened { index: 86 }) => Purpose::TR,
                    _ => return Err(Error::UnsupportedDerivationPath),
                };
                let _net = first_path.next();
                let account = first_path.next().ok_or(Error::InvalidDerivationPath)?;
                let account = if let ChildNumber::Hardened { index } = account {
                    *index
                } else {
                    return Err(Error::InvalidDerivationPath);
                };
                let change = first_path.next().ok_or(Error::InvalidDerivationPath)?;
                let change = if let ChildNumber::Normal { index } = change {
                    match index {
                        0 => false,
                        1 => true,
                        _ => return Err(Error::InvalidDerivationPath),
                    }
                } else {
                    return Err(Error::InvalidDerivationPath);
                };

                let descriptors = Descriptors::new(seed.clone(), network, Some(account))?;
                let descriptor = descriptors.get_by_purpose(purpose, change)?;
                descriptor.to_string()
            }
        };

        let mut wallet = Wallet::new(&descriptor, None, network, MemoryDatabase::default())?;

        let base_psbt = self.clone();
        let mut counter: usize = 0;

        for path in paths.into_iter() {
            let child_priv: ExtendedPrivKey = root.derive_priv(&SECP256K1, &path)?;
            let private_key: PrivateKey = PrivateKey::new(child_priv.private_key, network);
            let signer_ctx: SignerContext = match path.into_iter().next() {
                Some(ChildNumber::Hardened { index: 44 }) => SignerContext::Legacy,
                Some(ChildNumber::Hardened { index: 49 }) => SignerContext::Segwitv0,
                Some(ChildNumber::Hardened { index: 84 }) => SignerContext::Segwitv0,
                Some(ChildNumber::Hardened { index: 86 }) => SignerContext::Tap {
                    is_internal_key: false,
                },
                _ => return Err(Error::UnsupportedDerivationPath),
            };
            let signer = SignerWrapper::new(private_key, signer_ctx);
            wallet.add_signer(
                KeychainKind::External,
                SignerOrdering(counter),
                Arc::new(signer),
            );
            counter += 1;
        }

        for signer in custom_signers.into_iter() {
            wallet.add_signer(
                KeychainKind::External,
                SignerOrdering(counter),
                Arc::new(signer),
            );
            counter += 1;
        }

        let finalized = wallet.sign(self, SignOptions::default())?;

        if base_psbt != *self {
            Ok(finalized)
        } else {
            Err(Error::PsbtNotSigned)
        }
    }

    fn as_base64(&self) -> String {
        self.to_string()
    }
}

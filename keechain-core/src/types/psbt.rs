// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! PSBT

use core::fmt::{self, Debug};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use bdk::miniscript::descriptor::DescriptorKeyParseError;
use bdk::miniscript::Descriptor;
use bdk::signer::{SignerContext, SignerOrdering, SignerWrapper};
use bdk::{KeychainKind, SignOptions, Wallet};
use bitcoin::psbt::{self, PartiallySignedTransaction, PsbtParseError};
use bitcoin::secp256k1::{Secp256k1, Signing};
use bitcoin::{Network, PrivateKey};

use super::descriptors;
use crate::bips::bip32::{self, Bip32, ChildNumber, DerivationPath, ExtendedPrivKey, Fingerprint};
use crate::types::{Descriptors, Purpose, Seed};
use crate::util::base64;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Base64(base64::DecodeError),
    BIP32(bip32::Error),
    Psbt(psbt::Error),
    PsbtParse(PsbtParseError),
    Descriptors(descriptors::Error),
    DescriptorParse(DescriptorKeyParseError),
    Bdk(bdk::Error),
    BdkDescriptor(bdk::descriptor::DescriptorError),
    FileNotFound,
    UnsupportedDerivationPath,
    InvalidDerivationPath,
    NothingToSign,
    PsbtNotSigned,
    /// Negative fee
    NegativeFee,
    /// Integer overflow in fee calculation
    FeeOverflow,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "IO: {e}"),
            Self::Base64(e) => write!(f, "Base64: {e}"),
            Self::BIP32(e) => write!(f, "BIP32: {e}"),
            Self::Psbt(e) => write!(f, "Psbt: {e}"),
            Self::PsbtParse(e) => write!(f, "Psbt parse: {e}"),
            Self::Descriptors(e) => write!(f, "Descriptors: {e}"),
            Self::DescriptorParse(e) => write!(f, "Descriptor parse: {e}"),
            Self::Bdk(e) => write!(f, "Bdk: {e}"),
            Self::BdkDescriptor(e) => write!(f, "Bdk bescriptor: {e}"),
            Self::FileNotFound => write!(f, "File not found"),
            Self::UnsupportedDerivationPath => write!(f, "Unsupported derivation path"),
            Self::InvalidDerivationPath => write!(f, "Invalid derivation path"),
            Self::NothingToSign => write!(f, "Nothing to sign here"),
            Self::PsbtNotSigned => write!(f, "PSBT not signed"),
            Self::NegativeFee => write!(f, "Negative fee"),
            Self::FeeOverflow => write!(f, "Fee overflow"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Self::Base64(e)
    }
}

impl From<bip32::Error> for Error {
    fn from(e: bip32::Error) -> Self {
        Self::BIP32(e)
    }
}

impl From<psbt::Error> for Error {
    fn from(e: psbt::Error) -> Self {
        Self::Psbt(e)
    }
}

impl From<PsbtParseError> for Error {
    fn from(e: PsbtParseError) -> Self {
        Self::PsbtParse(e)
    }
}

impl From<descriptors::Error> for Error {
    fn from(e: descriptors::Error) -> Self {
        Self::Descriptors(e)
    }
}

impl From<DescriptorKeyParseError> for Error {
    fn from(e: DescriptorKeyParseError) -> Self {
        Self::DescriptorParse(e)
    }
}

impl From<bdk::Error> for Error {
    fn from(e: bdk::Error) -> Self {
        Self::Bdk(e)
    }
}

impl From<bdk::descriptor::DescriptorError> for Error {
    fn from(e: bdk::descriptor::DescriptorError) -> Self {
        Self::BdkDescriptor(e)
    }
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

    fn sign<C>(&mut self, seed: &Seed, network: Network, secp: &Secp256k1<C>) -> Result<bool, Error>
    where
        C: Signing,
    {
        self.sign_custom(seed, None, Vec::new(), false, network, secp)
    }

    fn sign_with_descriptor<C>(
        &mut self,
        seed: &Seed,
        descriptor: Descriptor<String>,
        use_tr_internal_key: bool,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<bool, Error>
    where
        C: Signing,
    {
        self.sign_custom(
            seed,
            Some(descriptor),
            Vec::new(),
            use_tr_internal_key,
            network,
            secp,
        )
    }

    fn sign_custom<C>(
        &mut self,
        seed: &Seed,
        descriptor: Option<Descriptor<String>>,
        custom_signers: Vec<SignerWrapper<PrivateKey>>,
        use_tr_internal_key: bool,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<bool, Error>
    where
        C: Signing;

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

    /// Calculates transaction fee.
    ///
    /// 'Fee' being the amount that will be paid for mining a transaction with the current inputs
    /// and outputs i.e., the difference in value of the total inputs and the total outputs.
    ///
    /// ## Errors
    ///
    /// - [`Error::MissingUtxo`] when UTXO information for any input is not present or is invalid.
    /// - [`Error::NegativeFee`] if calculated value is negative.
    /// - [`Error::FeeOverflow`] if an integer overflow occurs.
    fn fee(&self) -> Result<u64, Error>;
}

impl Psbt for PartiallySignedTransaction {
    fn from_base64<S>(psbt: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        Ok(PartiallySignedTransaction::from_str(&psbt.into())?)
    }

    fn sign_custom<C>(
        &mut self,
        seed: &Seed,
        descriptor: Option<Descriptor<String>>,
        custom_signers: Vec<SignerWrapper<PrivateKey>>,
        use_tr_internal_key: bool,
        network: Network,
        secp: &Secp256k1<C>,
    ) -> Result<bool, Error>
    where
        C: Signing,
    {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let root_fingerprint: Fingerprint = root.fingerprint(secp);

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

                let descriptors = Descriptors::new(seed.clone(), network, Some(account), secp)?;
                let descriptor = descriptors.get_by_purpose(purpose, change)?;
                descriptor.to_string()
            }
        };

        let mut wallet = Wallet::new_no_persist(&descriptor, None, network)?;

        let base_psbt = self.clone();
        let mut counter: usize = 0;

        for path in paths.into_iter() {
            let child_priv: ExtendedPrivKey = root.derive_priv(secp, &path)?;
            let private_key: PrivateKey = PrivateKey::new(child_priv.private_key, network);
            let signer_ctx: SignerContext = match path.into_iter().next() {
                Some(ChildNumber::Hardened { index: 44 }) => SignerContext::Legacy,
                Some(ChildNumber::Hardened { index: 49 }) => SignerContext::Segwitv0,
                Some(ChildNumber::Hardened { index: 84 }) => SignerContext::Segwitv0,
                Some(ChildNumber::Hardened { index: 86 }) => SignerContext::Tap {
                    is_internal_key: use_tr_internal_key,
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

    /// Calculates transaction fee.
    ///
    /// 'Fee' being the amount that will be paid for mining a transaction with the current inputs
    /// and outputs i.e., the difference in value of the total inputs and the total outputs.
    ///
    /// ## Errors
    ///
    /// - [`Error::MissingUtxo`] when UTXO information for any input is not present or is invalid.
    /// - [`Error::NegativeFee`] if calculated value is negative.
    /// - [`Error::FeeOverflow`] if an integer overflow occurs.
    fn fee(&self) -> Result<u64, Error> {
        let mut inputs: u64 = 0;
        for utxo in self.iter_funding_utxos() {
            inputs = inputs.checked_add(utxo?.value).ok_or(Error::FeeOverflow)?;
        }
        let mut outputs: u64 = 0;
        for out in &self.unsigned_tx.output {
            outputs = outputs.checked_add(out.value).ok_or(Error::FeeOverflow)?;
        }
        inputs.checked_sub(outputs).ok_or(Error::NegativeFee)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bip39::Mnemonic;
    use bitcoin::Network;

    use super::*;
    use crate::types::Seed;

    const NETWORK: Network = Network::Testnet;

    #[test]
    fn test_psbt_sign() {
        let secp = Secp256k1::new();
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let seed = Seed::new::<&str>(mnemonic, None);
        let mut psbt = PartiallySignedTransaction::from_base64("cHNidP8BAFICAAAAATjFB9Xkau6+MTmNTT9GN6i299X9n9MSQhVVMVegw8qOAAAAAAD9////AcAHAAAAAAAAFgAUAhYIdK3p2Bvf/ZnzIYQcWWZkxCJ4HiUATwEENYfPA+UBpeaAAAAAVd9MbQ78ZD7Ie5K8FXctxNRCrS4DNFhPiSzC2CpygWICsOropyXycdL0H0uI5TUbJL1w8/detLdnP5WxGGUZ+5UQm/Q1S1QAAIABAACAAAAAgAABAHECAAAAAYqdaqOD/k1QaGShhL4ilryMhXgOJu+cFcKFAUMZQ+wrAAAAAAD9////Ai4IAAAAAAAAFgAUqjLdU2PqfvD/lSvnNLJZ0ab4kUPxCQAAAAAAABYAFO9WcMNPGiI5MjypE7Ku0dT1LOgRI9wkAAEBHy4IAAAAAAAAFgAUqjLdU2PqfvD/lSvnNLJZ0ab4kUMBAwQBAAAAIgYCyh1DqpGE/SatxQ86lKeUBXZ1BGpZuwNnGiGq9pDdTbkYm/Q1S1QAAIABAACAAAAAgAAAAAAAAAAAAAA=").unwrap();
        let finalized = psbt.sign(&seed, NETWORK, &secp).unwrap();
        assert!(finalized);
    }

    #[test]
    fn test_psbt_sign_custom_internal() {
        let secp = Secp256k1::new();
        let descriptor: Descriptor<String> = Descriptor::from_str("tr([9bf4354b/86'/1'/784923']tpubDCT8uwnkZj7woaY71Xr5hU7Wvjr7B1BXJEpwMzzDLd1H6HLnKTiaLPtt6ZfEizDMwdQ8PT8JCmKbB4ESVXTkCzv51oxhJhX5FLBvkeN9nJ3/0/*,pk([7356e457/86'/1'/784923']tpubDCvLwbJPseNux9EtPbrbA2tgDayzptK4HNkky14Cw6msjHuqyZCE88miedZD86TZUb29Rof3sgtREU4wtzofte7QDSWDiw8ZU6ZYHmAxY9d/0/*))#rs0udsfg").unwrap();
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let seed = Seed::new::<&str>(mnemonic, None);
        let mut psbt = PartiallySignedTransaction::from_base64("cHNidP8BAIABAAAAAQiqsV3pVy3i3mOXb44eSY6YXfyBJJquLJUFOQgKxqogAQAAAAD9////ApcWAAAAAAAAGXapFFnK2lAxTIKeGfWneG+O4NSYf0KdiKysDAAAAAAAACJRIDah9WL9RrG8cBtYLPY/dqsOd9+Ysh7+hNnInepPfCUoKTclAAABASvmIwAAAAAAACJRIIFkFWTG5s8O4M/FVct0eYcA0ayNYYMfdUK3VDHm3PNNIhXAAMzzAr/xU1CxCRn2xLf6Vk7deJJ1P2IphMFQkGwGZNwjIFSh53RXgXULuDjlB82aLiF9LkqzhtrTHbwF5MJP9JNyrMAhFlSh53RXgXULuDjlB82aLiF9LkqzhtrTHbwF5MJP9JNyOQETYY0ojn8xo/xlOd4vxPBtGqXOW/RgxpD1azdzLllueXNW5FdWAACAAQAAgBv6C4AAAAAAAAAAACEWAMzzAr/xU1CxCRn2xLf6Vk7deJJ1P2IphMFQkGwGZNwZAJv0NUtWAACAAQAAgBv6C4AAAAAAAAAAAAEXIADM8wK/8VNQsQkZ9sS3+lZO3XiSdT9iKYTBUJBsBmTcARggE2GNKI5/MaP8ZTneL8TwbRqlzlv0YMaQ9Ws3cy5ZbnkAAAEFIMyrxjur6FZA49b3vxbW2gGoFCVIDqhp4WQ8eJq6uV9EAQYlAMAiIFQ0gIXoLoC1Uk+d9i2t+6KirZ4znJISAZS7NkP7DSBbrCEHzKvGO6voVkDj1ve/FtbaAagUJUgOqGnhZDx4mrq5X0QZAJv0NUtWAACAAQAAgBv6C4AAAAAAAQAAACEHVDSAhegugLVST532La37oqKtnjOckhIBlLs2Q/sNIFs5ARpaIl7upiRp2Mj47BtMoV8ZSitR752q1zy5u5ZgWQ7Lc1bkV1YAAIABAACAG/oLgAAAAAABAAAAAA==").unwrap();
        let finalized = psbt
            .sign_custom(&seed, Some(descriptor), Vec::new(), true, NETWORK, &secp)
            .unwrap();
        assert!(finalized);
    }
}

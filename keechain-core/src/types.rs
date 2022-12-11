// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use bdk::database::MemoryDatabase;
use bdk::keys::bip39::Mnemonic;
use bdk::keys::DescriptorSecretKey;
use bdk::miniscript::Descriptor;
use bdk::wallet::AddressIndex;
use bdk::{SignOptions, Wallet};
use bitcoin::psbt::PartiallySignedTransaction;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, Fingerprint};
use bitcoin::{Address, Network, TxOut};
use clap::ValueEnum;
use num_format::{Locale, ToFormattedString};
use prettytable::format::FormatBuilder;
use prettytable::{row, Table};
use serde::{Deserialize, Serialize};

use crate::crypto::aes::{self, Aes256Encryption};
use crate::error::{Error, Result};
use crate::util::bip::bip32::Bip32RootKey;
use crate::util::{self, convert};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Seed {
    mnemonic: Mnemonic,
    passphrase: Option<String>,
}

impl Seed {
    pub fn new<S>(mnemonic: Mnemonic, passphrase: Option<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            mnemonic,
            passphrase: passphrase.map(|p| p.into()),
        }
    }

    pub fn mnemonic(&self) -> Mnemonic {
        self.mnemonic.clone()
    }

    pub fn passphrase(&self) -> Option<String> {
        self.passphrase.clone()
    }

    pub fn to_bytes(&self) -> [u8; 64] {
        self.mnemonic
            .to_seed(self.passphrase.clone().unwrap_or_default())
    }

    pub fn to_hex(&self) -> String {
        convert::bytes_to_hex(self.to_bytes().to_vec())
    }
}

impl Bip32RootKey for Seed {
    type Err = Error;
    fn to_bip32_root_key(&self, network: Network) -> Result<ExtendedPrivKey, Self::Err> {
        Ok(ExtendedPrivKey::new_master(network, &self.to_bytes())?)
    }

    fn fingerprint(&self, network: Network) -> Result<Fingerprint, Self::Err> {
        let root: ExtendedPrivKey = self.to_bip32_root_key(network)?;
        let secp = Secp256k1::new();
        Ok(root.fingerprint(&secp))
    }
}

impl Aes256Encryption for Seed {
    type Err = Error;
    fn encrypt<K>(&self, key: K) -> Result<Vec<u8>, Self::Err>
    where
        K: AsRef<[u8]>,
    {
        let serialized_seed: Vec<u8> = util::serialize(self)?;
        Ok(aes::encrypt(key, &serialized_seed))
    }

    fn decrypt<K>(key: K, content: &[u8]) -> Result<Self, Self::Err>
    where
        K: AsRef<[u8]>,
    {
        match aes::decrypt(key, content) {
            Ok(data) => util::deserialize(data),
            Err(aes::Error::WrongBlockMode) => Err(Error::Generic(
                "Impossible to decrypt file: invalid password or content".to_string(),
            )),
            Err(e) => Err(Error::Aes(e)),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
#[repr(u8)]
pub enum WordCount {
    #[clap(name = "12")]
    W12 = 12,
    #[clap(name = "18")]
    W18 = 18,
    #[clap(name = "24")]
    W24 = 24,
}

impl WordCount {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Clone, Copy)]
pub struct Index(u32);

impl Index {
    pub fn new(index: u32) -> Result<Self> {
        if index & (1 << 31) == 0 {
            Ok(Self(index))
        } else {
            Err(Error::Generic("Invalid index".to_string()))
        }
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl FromStr for Index {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let index: u32 = s
            .parse()
            .map_err(|_| Error::Parse("Impossible to parse index".to_string()))?;
        Self::new(index)
    }
}

impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}

#[derive(Debug, Clone)]
pub struct Descriptors {
    pub external: Vec<Descriptor<String>>,
    pub internal: Vec<Descriptor<String>>,
}

pub struct Secrets {
    pub entropy: String,
    pub mnemonic: Mnemonic,
    pub passphrase: Option<String>,
    pub seed_hex: String,
    pub network: Network,
    pub root_key: ExtendedPrivKey,
    pub fingerprint: Fingerprint,
}

impl Secrets {
    pub fn new(seed: Seed, network: Network) -> Result<Self> {
        let secp = Secp256k1::new();
        let mnemonic: Mnemonic = seed.mnemonic();
        let root_key: ExtendedPrivKey = seed.to_bip32_root_key(network)?;

        Ok(Self {
            entropy: convert::bytes_to_hex(mnemonic.to_entropy()),
            mnemonic,
            passphrase: seed.passphrase(),
            seed_hex: seed.to_hex(),
            network,
            root_key,
            fingerprint: root_key.fingerprint(&secp),
        })
    }

    pub fn print(&self) {
        let mut table = Table::new();

        table.add_row(row![
            format!("Entropy ({} bits)", self.entropy.len() / 2 * 8),
            self.entropy
        ]);
        table.add_row(row!["Mnemonic (BIP39)", self.mnemonic]);

        if let Some(passphrase) = &self.passphrase {
            table.add_row(row!["Passphrase (BIP39)", passphrase]);
        }

        table.add_row(row!["Seed HEX (BIP39)", self.seed_hex]);
        table.add_row(row!["Network", self.network]);
        table.add_row(row!["Root Key (BIP32)", self.root_key]);
        table.add_row(row!["Fingerprint (BIP32)", self.fingerprint]);

        table.printstd();
    }
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

    pub fn sign(&mut self, seed: &Seed) -> Result<bool> {
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
        }

        if paths.is_empty() {
            return Err(Error::Generic("Nothing to sign here.".to_string()));
        }

        let mut finalized: bool = false;

        for path in paths.into_iter() {
            let child_priv: ExtendedPrivKey = root.derive_priv(&secp, &path)?;
            let desc = DescriptorSecretKey::from_str(&child_priv.to_string())
                .map_err(|e| Error::Parse(format!("Impossible to parse descriptor: {}", e)))?;
            let descriptor = match path.into_iter().next() {
                Some(ChildNumber::Hardened { index: 44 }) => format!("pkh({})", desc),
                Some(ChildNumber::Hardened { index: 49 }) => format!("sh(wpkh({}))", desc),
                Some(ChildNumber::Hardened { index: 84 }) => format!("wpkh({})", desc),
                Some(ChildNumber::Hardened { index: 86 }) => format!("tr({})", desc),
                _ => return Err(Error::Generic("Unsupported derivation path".to_string())),
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

    pub fn save_to_file<P>(&self, path: P) -> Result<()>
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

    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        Ok(base64::decode(self.as_base64())?)
    }

    fn output_table_row(&self, output: &TxOut) -> Result<String> {
        let mut table = Table::new();
        let format = FormatBuilder::new()
            .column_separator('|')
            .padding(0, 0)
            .build();
        table.set_format(format);
        table.add_row(row![
            format!(
                "{} ",
                Address::from_script(&output.script_pubkey, self.network)
                    .map_err(|e| Error::Generic(e.to_string()))?
            ),
            format!(" {} sat", output.value.to_formatted_string(&Locale::fr))
        ]);
        Ok(table.to_string())
    }

    pub fn print(&self) -> Result<()> {
        let tx = self.psbt.clone().extract_tx();
        let inputs_len: usize = tx.input.len();
        let outputs_len: usize = tx.output.len();

        let mut table = Table::new();

        table.set_titles(row![
            format!("Inputs ({})", inputs_len),
            format!("Outputs ({})", outputs_len)
        ]);

        if inputs_len >= outputs_len {
            for (index, input) in tx.input.iter().enumerate() {
                let input = format!("{}", input.previous_output);
                if let Some(output) = tx.output.get(index) {
                    table.add_row(row![input, self.output_table_row(output)?]);
                } else {
                    table.add_row(row![input, ""]);
                }
            }
        } else {
            for (index, output) in tx.output.iter().enumerate() {
                let output = self.output_table_row(output)?;
                if let Some(input) = tx.input.get(index) {
                    table.add_row(row![format!("{}", input.previous_output), output]);
                } else {
                    table.add_row(row!["", output]);
                }
            }
        }

        table.printstd();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed() {
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase).unwrap();
        assert_eq!(&seed.to_hex(), "fb826595a0d679f5e9f8c799bd1decb8dc2ad3fb4e39a1ffaa4708a150e0e81ae55d3f340a188cd6188a2b76601aeae16945b36ae0ecfced9645029796c33713")
    }

    #[test]
    fn test_index() {
        let index = Index::new(2345).unwrap();
        assert_eq!(index.as_u32(), 2345);
        assert!(Index::new(2147483647).is_ok());
        assert!(Index::new(2147483648).is_err());
    }
}

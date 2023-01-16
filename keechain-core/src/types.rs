// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use bdk::database::MemoryDatabase;
use bdk::keys::bip39::Mnemonic;
use bdk::miniscript::descriptor::{Descriptor, DescriptorSecretKey};
use bdk::wallet::AddressIndex;
use bdk::{SignOptions, Wallet};
use bitcoin::psbt::PartiallySignedTransaction;
use bitcoin::secp256k1::Secp256k1;
use bitcoin::util::bip32::{
    ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey, Fingerprint,
};
use bitcoin::Network;
use clap::ValueEnum;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::util::bip::bip32::{self, Bip32RootKey};
use crate::util::convert;
use crate::util::slip::slip132::ToSlip132;

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

    pub fn from_mnemonic(mnemonic: Mnemonic) -> Self {
        Self {
            mnemonic,
            passphrase: None,
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

impl Default for WordCount {
    fn default() -> Self {
        Self::W24
    }
}

impl WordCount {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Clone, Copy, Default)]
pub struct Index(u32);

pub const MAX_INDEX: u32 = 0x80000000;

impl Index {
    pub fn new(index: u32) -> Result<Self> {
        if index < MAX_INDEX {
            Ok(Self(index))
        } else {
            Err(Error::Generic("Invalid index".to_string()))
        }
    }

    pub fn increment(&mut self) {
        if self.0 >= MAX_INDEX {
            self.0 = 0;
        } else {
            self.0 += 1;
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

    pub fn from_base64<S>(psbt: S, network: Network) -> Result<Self>
    where
        S: Into<String>,
    {
        Ok(Psbt::new(
            PartiallySignedTransaction::from_str(&psbt.into())
                .map_err(|e| Error::Parse(format!("Impossible to parse PSBT: {}", e)))?,
            network,
        ))
    }

    pub fn from_file<P>(path: P, network: Network) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let psbt_file = path.as_ref();
        if !psbt_file.exists() && !psbt_file.is_file() {
            return Err(Error::Generic("PSBT file not found.".to_string()));
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
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
#[repr(u8)]
pub enum ElectrumExportSupportedScripts {
    /// P2PKH (BIP44)
    Legacy = 44,
    /// P2SHWPKH (BIP49)
    Segwit = 49,
    /// P2WPKH (BIP84)
    NativeSegwit = 84,
}

impl Default for ElectrumExportSupportedScripts {
    fn default() -> Self {
        Self::NativeSegwit
    }
}

impl ElectrumExportSupportedScripts {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}

impl fmt::Display for ElectrumExportSupportedScripts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Legacy => write!(f, "legacy"),
            Self::Segwit => write!(f, "segwit"),
            Self::NativeSegwit => write!(f, "native-segwit"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BitcoinCoreDescriptor {
    timestamp: String,
    active: bool,
    desc: Descriptor<String>,
    internal: bool,
}

impl BitcoinCoreDescriptor {
    pub fn new(desc: Descriptor<String>, internal: bool) -> Self {
        Self {
            timestamp: String::from("now"),
            active: true,
            desc,
            internal,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ElectrumJsonKeystore {
    xpub: String,
    #[serde(skip)]
    fingerprint: Fingerprint,
    root_fingerprint: Fingerprint,
    #[serde(rename = "type")]
    keystore_type: String,
    derivation: DerivationPath,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ElectrumJsonWallet {
    keystore: ElectrumJsonKeystore,
    wallet_type: String,
    use_encryption: bool,
    seed_version: u32,
}

impl ElectrumJsonWallet {
    pub fn new(
        seed: Seed,
        network: Network,
        script: ElectrumExportSupportedScripts,
        account: Option<u32>,
    ) -> Result<Self> {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let secp = Secp256k1::new();
        let path: DerivationPath = bip32::account_extended_path(script.as_u32(), network, account)?;
        let pubkey: ExtendedPubKey =
            ExtendedPubKey::from_priv(&secp, &root.derive_priv(&secp, &path)?);

        Ok(Self {
            keystore: ElectrumJsonKeystore {
                xpub: pubkey.to_slip132(&path)?,
                fingerprint: pubkey.fingerprint(),
                root_fingerprint: root.fingerprint(&secp),
                keystore_type: String::from("bip32"),
                derivation: path,
            },
            wallet_type: String::from("standard"),
            use_encryption: false,
            seed_version: 20,
        })
    }

    pub fn save_to_file<P>(&self, path: P) -> Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        let file_name: String = format!("keechain-{}.json", self.keystore.fingerprint);
        let path: PathBuf = path.as_ref().join(file_name);
        let mut file: File = File::options().create(true).write(true).open(&path)?;
        file.write_all(&serde_json::to_vec(self)?)?;
        Ok(path)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WasabiJsonWallet {
    #[serde(rename = "ExtPubKey")]
    xpub: ExtendedPubKey,
    #[serde(rename = "MasterFingerprint")]
    root_fingerprint: Fingerprint,
}

impl WasabiJsonWallet {
    pub fn new(seed: Seed, network: Network) -> Result<Self> {
        let root: ExtendedPrivKey = seed.to_bip32_root_key(network)?;
        let secp = Secp256k1::new();
        let path: DerivationPath = bip32::account_extended_path(84, network, None)?;
        let pubkey: ExtendedPubKey =
            ExtendedPubKey::from_priv(&secp, &root.derive_priv(&secp, &path)?);

        Ok(Self {
            xpub: pubkey,
            root_fingerprint: root.fingerprint(&secp),
        })
    }

    pub fn save_to_file<P>(&self, path: P) -> Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        let file_name: String = format!("keechain-wasabi-{}.json", self.xpub.fingerprint());
        let path: PathBuf = path.as_ref().join(file_name);
        let mut file: File = File::options().create(true).write(true).open(&path)?;
        file.write_all(&serde_json::to_vec(self)?)?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed() {
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);
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

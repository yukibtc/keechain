// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! BIP85
//!
//! <https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki>

use core::fmt;

use bip39::Mnemonic;
use bitcoin::hashes::hmac::{Hmac, HmacEngine};
use bitcoin::hashes::{sha512, Hash, HashEngine};
use bitcoin::secp256k1::{Secp256k1, Signing};
use bitcoin::util::bip32;
use bitcoin::Network;

use super::bip32::{Bip32, ChildNumber, DerivationPath, ExtendedPrivKey};
use crate::types::{Index, WordCount};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    BIP32(bip32::Error),
    BIP39(bip39::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BIP32(e) => write!(f, "BIP32: {e}"),
            Self::BIP39(e) => write!(f, "BIP39: {e}"),
        }
    }
}

impl From<bip32::Error> for Error {
    fn from(e: bip32::Error) -> Self {
        Self::BIP32(e)
    }
}

impl From<bip39::Error> for Error {
    fn from(e: bip39::Error) -> Self {
        Self::BIP39(e)
    }
}

pub trait FromBip85: Sized {
    fn from_bip85<C>(
        root: &ExtendedPrivKey,
        word_count: WordCount,
        index: Index,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Error>
    where
        C: Signing;
}

impl FromBip85 for Mnemonic {
    fn from_bip85<C>(
        root: &ExtendedPrivKey,
        word_count: WordCount,
        index: Index,
        secp: &Secp256k1<C>,
    ) -> Result<Self, Error>
    where
        C: Signing,
    {
        let word_count: u32 = word_count.as_u32();
        let path: Vec<ChildNumber> = vec![
            ChildNumber::from_hardened_idx(83696968)?,
            ChildNumber::from_hardened_idx(39)?,
            ChildNumber::from_hardened_idx(0)?,
            ChildNumber::from_hardened_idx(word_count)?,
            ChildNumber::from_hardened_idx(index.as_u32())?,
        ];
        let path: DerivationPath = DerivationPath::from(path);
        let derived: ExtendedPrivKey = root.derive_priv(secp, &path)?;

        let mut h = HmacEngine::<sha512::Hash>::new(b"bip-entropy-from-k");
        h.input(&derived.private_key.secret_bytes());
        let data: [u8; 64] = Hmac::from_engine(h).into_inner();
        let len: u32 = word_count * 4 / 3;
        Ok(Mnemonic::from_entropy(&data[0..len as usize])?)
    }
}

pub trait Bip85: Sized + Bip32
where
    Error: From<<Self as Bip32>::Err>,
{
    /// Derive BIP85 mnemonic
    ///
    /// <https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki>
    fn derive_bip85_mnemonic<C>(
        &self,
        word_count: WordCount,
        index: Index,
        secp: &Secp256k1<C>,
    ) -> Result<Mnemonic, Error>
    where
        C: Signing,
    {
        let root: ExtendedPrivKey = self.to_bip32_root_key(Network::Bitcoin)?;
        Mnemonic::from_bip85(&root, word_count, index, secp)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::Network;

    use super::*;
    use crate::types::{Index, Seed, WordCount};

    const NETWORK: Network = Network::Testnet;

    #[test]
    fn test_from_bip85() {
        let secp = Secp256k1::new();
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);

        let root = ExtendedPrivKey::new_master(NETWORK, &seed.to_bytes()).unwrap();

        // Words: 12
        // Index: 0
        assert_eq!(
            Mnemonic::from_bip85(&root, WordCount::W12, Index::new(0).unwrap(), &secp)
                .unwrap()
                .to_string(),
            "gap gun smooth leader muscle renew impulse hundred twin enact fetch zoo".to_string()
        );

        // Words: 12
        // Index: 1
        assert_eq!(
            Mnemonic::from_bip85(&root, WordCount::W12, Index::new(1).unwrap(), &secp)
                .unwrap()
                .to_string(),
            "join siren history age snack dial initial raise kick enter vintage rabbit".to_string()
        );

        // Words: 24
        // Index: 57
        assert_eq!(
            Mnemonic::from_bip85(&root, WordCount::W24, Index::new(57).unwrap(), &secp)
                .unwrap()
                .to_string(),
            "this supply project flush south sport acid focus damp pulp hundred convince ramp mandate picnic area bracket group pact piano coconut cigar decline actress".to_string()
        );

        // Test wrong seed
        assert_ne!(
            Mnemonic::from_bip85(&root, WordCount::W12, Index::new(12).unwrap(), &secp)
                .unwrap()
                .to_string(),
            "pride drama job inform cross recall vapor lake weasel basket curve pencil".to_string()
        )
    }

    #[test]
    fn test_to_bip85() {
        let secp = Secp256k1::new();
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);

        // Words: 12
        // Index: 0
        assert_eq!(
            seed.derive_bip85_mnemonic(WordCount::W12, Index::new(0).unwrap(), &secp)
                .unwrap()
                .to_string(),
            "gap gun smooth leader muscle renew impulse hundred twin enact fetch zoo".to_string()
        );

        // Words: 12
        // Index: 1
        assert_eq!(
            seed.derive_bip85_mnemonic(WordCount::W12, Index::new(1).unwrap(), &secp)
                .unwrap()
                .to_string(),
            "join siren history age snack dial initial raise kick enter vintage rabbit".to_string()
        );

        // Words: 24
        // Index: 57
        assert_eq!(
            seed.derive_bip85_mnemonic(WordCount::W24, Index::new(57).unwrap(), &secp)
                .unwrap()
                .to_string(),
            "this supply project flush south sport acid focus damp pulp hundred convince ramp mandate picnic area bracket group pact piano coconut cigar decline actress".to_string()
        );

        // Test wrong seed
        assert_ne!(
            seed.derive_bip85_mnemonic(WordCount::W12, Index::new(12).unwrap(), &secp)
                .unwrap()
                .to_string(),
            "pride drama job inform cross recall vapor lake weasel basket curve pencil".to_string()
        )
    }

    #[test]
    fn test_eq_bip85_result() {
        let secp = Secp256k1::new();
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);

        let root = ExtendedPrivKey::new_master(Network::Testnet, &seed.to_bytes()).unwrap();
        assert_eq!(
            seed.derive_bip85_mnemonic(WordCount::W12, Index::new(0).unwrap(), &secp)
                .unwrap(),
            Mnemonic::from_bip85(&root, WordCount::W12, Index::new(0).unwrap(), &secp).unwrap()
        );

        let root = ExtendedPrivKey::new_master(Network::Regtest, &seed.to_bytes()).unwrap();
        assert_eq!(
            seed.derive_bip85_mnemonic(WordCount::W24, Index::new(4).unwrap(), &secp)
                .unwrap(),
            Mnemonic::from_bip85(&root, WordCount::W24, Index::new(4).unwrap(), &secp).unwrap()
        );
    }
}

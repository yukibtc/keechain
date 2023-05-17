// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! BIP85
//!
//! <https://github.com/bitcoin/bips/blob/master/bip-0085.mediawiki>

use bip39::Mnemonic;
use bitcoin::hashes::hmac::{Hmac, HmacEngine};
use bitcoin::hashes::{sha512, Hash, HashEngine};
use bitcoin::Network;

use super::bip32::{Bip32, ChildNumber, DerivationPath, ExtendedPrivKey};
use crate::types::{Index, WordCount};
use crate::SECP256K1;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    BIP32(#[from] bitcoin::util::bip32::Error),
    #[error(transparent)]
    BIP39(#[from] bip39::Error),
}

pub trait FromBip85: Sized {
    fn from_bip85(
        root: &ExtendedPrivKey,
        word_count: WordCount,
        index: Index,
    ) -> Result<Self, Error>;
}

impl FromBip85 for Mnemonic {
    fn from_bip85(
        root: &ExtendedPrivKey,
        word_count: WordCount,
        index: Index,
    ) -> Result<Self, Error> {
        let word_count: u32 = word_count.as_u32();
        let path: Vec<ChildNumber> = vec![
            ChildNumber::from_hardened_idx(83696968)?,
            ChildNumber::from_hardened_idx(39)?,
            ChildNumber::from_hardened_idx(0)?,
            ChildNumber::from_hardened_idx(word_count)?,
            ChildNumber::from_hardened_idx(index.as_u32())?,
        ];
        let path: DerivationPath = DerivationPath::from(path);
        let derived: ExtendedPrivKey = root.derive_priv(&SECP256K1, &path)?;

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
    fn derive_bip85_mnemonic(
        &self,
        word_count: WordCount,
        index: Index,
    ) -> Result<Mnemonic, Error> {
        let root: ExtendedPrivKey = self.to_bip32_root_key(Network::Bitcoin)?;
        Mnemonic::from_bip85(&root, word_count, index)
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
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);

        let root = ExtendedPrivKey::new_master(NETWORK, &seed.to_bytes()).unwrap();

        // Words: 12
        // Index: 0
        assert_eq!(
            Mnemonic::from_bip85(&root, WordCount::W12, Index::new(0).unwrap())
                .unwrap()
                .to_string(),
            "gap gun smooth leader muscle renew impulse hundred twin enact fetch zoo".to_string()
        );

        // Words: 12
        // Index: 1
        assert_eq!(
            Mnemonic::from_bip85(&root, WordCount::W12, Index::new(1).unwrap())
                .unwrap()
                .to_string(),
            "join siren history age snack dial initial raise kick enter vintage rabbit".to_string()
        );

        // Words: 24
        // Index: 57
        assert_eq!(
            Mnemonic::from_bip85(&root, WordCount::W24, Index::new(57).unwrap())
                .unwrap()
                .to_string(),
            "this supply project flush south sport acid focus damp pulp hundred convince ramp mandate picnic area bracket group pact piano coconut cigar decline actress".to_string()
        );

        // Test wrong seed
        assert_ne!(
            Mnemonic::from_bip85(&root, WordCount::W12, Index::new(12).unwrap())
                .unwrap()
                .to_string(),
            "pride drama job inform cross recall vapor lake weasel basket curve pencil".to_string()
        )
    }

    #[test]
    fn test_to_bip85() {
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);

        // Words: 12
        // Index: 0
        assert_eq!(
            seed.derive_bip85_mnemonic(WordCount::W12, Index::new(0).unwrap())
                .unwrap()
                .to_string(),
            "gap gun smooth leader muscle renew impulse hundred twin enact fetch zoo".to_string()
        );

        // Words: 12
        // Index: 1
        assert_eq!(
            seed.derive_bip85_mnemonic(WordCount::W12, Index::new(1).unwrap())
                .unwrap()
                .to_string(),
            "join siren history age snack dial initial raise kick enter vintage rabbit".to_string()
        );

        // Words: 24
        // Index: 57
        assert_eq!(
            seed.derive_bip85_mnemonic(WordCount::W24, Index::new(57).unwrap())
                .unwrap()
                .to_string(),
            "this supply project flush south sport acid focus damp pulp hundred convince ramp mandate picnic area bracket group pact piano coconut cigar decline actress".to_string()
        );

        // Test wrong seed
        assert_ne!(
            seed.derive_bip85_mnemonic(WordCount::W12, Index::new(12).unwrap())
                .unwrap()
                .to_string(),
            "pride drama job inform cross recall vapor lake weasel basket curve pencil".to_string()
        )
    }

    #[test]
    fn test_eq_bip85_result() {
        let mnemonic = Mnemonic::from_str("easy uncover favorite crystal bless differ energy seat ecology match carry group refuse together chat observe hidden glad brave month diesel sustain depth salt").unwrap();
        let passphrase: Option<&str> = Some("mypassphrase");
        let seed = Seed::new(mnemonic, passphrase);

        let root = ExtendedPrivKey::new_master(Network::Testnet, &seed.to_bytes()).unwrap();
        assert_eq!(
            seed.derive_bip85_mnemonic(WordCount::W12, Index::new(0).unwrap())
                .unwrap(),
            Mnemonic::from_bip85(&root, WordCount::W12, Index::new(0).unwrap()).unwrap()
        );

        let root = ExtendedPrivKey::new_master(Network::Regtest, &seed.to_bytes()).unwrap();
        assert_eq!(
            seed.derive_bip85_mnemonic(WordCount::W24, Index::new(4).unwrap())
                .unwrap(),
            Mnemonic::from_bip85(&root, WordCount::W24, Index::new(4).unwrap()).unwrap()
        );
    }
}

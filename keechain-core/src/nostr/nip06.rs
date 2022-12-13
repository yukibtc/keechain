// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey};
use bitcoin::Network;

use crate::error::Result;
use crate::types::Seed;
use crate::util::bip::bip32::Bip32RootKey;

/// Derive SecretKey from mnemonics (BIP39 - ENGLISH wordlist).
pub fn derive_secret_key_from_seed(seed: Seed) -> Result<SecretKey> {
    let secp = Secp256k1::new();
    let root_key: ExtendedPrivKey = seed.to_bip32_root_key(Network::Bitcoin)?;
    let path: DerivationPath = DerivationPath::from_str("m/44'/1237'/0'/0/0")?;
    let child_xprv: ExtendedPrivKey = root_key.derive_priv(&secp, &path)?;
    Ok(child_xprv.private_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nostr::ToBech32;
    use bdk::keys::bip39::Mnemonic;
    use std::str::FromStr;

    #[test]
    fn test_nip06() -> Result<()> {
        let mnemonic = Mnemonic::from_str("equal dragon fabric refuse stable cherry smoke allow alley easy never medal attend together lumber movie what sad siege weather matrix buffalo state shoot")?;
        let seed = Seed::new::<String>(mnemonic, None);
        assert_eq!(
            derive_secret_key_from_seed(seed)?.to_bech32()?,
            "nsec1q6vjgxdgl6ppmkx7q02vxqrpf687a7674ymtwmufjaku4n52a0hq9glmaf".to_string()
        );

        Ok(())
    }
}

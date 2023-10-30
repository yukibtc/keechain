// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! BIP43
//!
//! <https://github.com/bitcoin/bips/blob/master/bip-0043.mediawiki>

use bdk::bitcoin::Network;

use super::bip32::{self, DerivationPath, Error};
use super::bip48::{self, ScriptType};

/// Derivation path purpose
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Purpose {
    /// BIP44 - P2PKH
    ///
    /// <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
    BIP44,
    /// BIP48 - Multi-Script Hierarchy for Multi-Sig Wallets
    ///
    /// <https://github.com/bitcoin/bips/blob/master/bip-0048.mediawiki>
    BIP48 { script: ScriptType },
    /// BIP49 - P2SH-WPKH
    ///
    /// <https://github.com/bitcoin/bips/blob/master/bip-0049.mediawiki>
    BIP49,
    /// BIP84 - P2WPKH
    ///
    /// <https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki>
    BIP84,
    /// BIP86 - P2TR
    ///
    /// <https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki>
    BIP86,
}

impl Purpose {
    pub fn to_account_extended_path(
        &self,
        network: Network,
        account: Option<u32>,
    ) -> Result<DerivationPath, Error> {
        match self {
            Self::BIP44 | Self::BIP49 | Self::BIP84 | Self::BIP86 => Ok(
                bip32::account_extended_path(self.as_u32(), network, account)?,
            ),
            Self::BIP48 { script } => Ok(bip48::account_extended_path(network, account, *script)?),
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            Self::BIP44 => 44,
            Self::BIP48 { .. } => 48,
            Self::BIP49 => 49,
            Self::BIP84 => 84,
            Self::BIP86 => 86,
        }
    }
}

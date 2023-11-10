// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

//! BIP43
//!
//! <https://github.com/bitcoin/bips/blob/master/bip-0043.mediawiki>

use std::{fmt, fmt::Display, str::FromStr};

use bdk::bitcoin::Network;

use super::bip32::{self, DerivationPath};
use super::bip48::{self, ScriptType};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
pub enum Error {
    UnknownPurpose,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownPurpose => write!(f, "unknown purpose"),
        }
    }
}

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
    ) -> Result<DerivationPath, bip32::Error> {
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

impl Display for Purpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let purpose: &str = match self {
            Purpose::BIP44 => "bip44",
            Purpose::BIP49 => "bip49",
            Purpose::BIP84 => "bip84",
            Purpose::BIP86 => "bip86",
            Purpose::BIP48 { script } => match script {
                ScriptType::P2SHWSH => "bip48_1",
                ScriptType::P2WSH => "bip48_2",
                ScriptType::P2TR => "bip48_3",
            },
        };
        write!(f, "{}", purpose)
    }
}

impl FromStr for Purpose {
    type Err = Error;
    fn from_str(purpose: &str) -> Result<Self, Self::Err> {
        let purpose = match purpose {
            "bip44" => Purpose::BIP44,
            "bip49" => Purpose::BIP49,
            "bip84" => Purpose::BIP84,
            "bip86" => Purpose::BIP86,
            "bip48_1" => Purpose::BIP48 {
                script: ScriptType::P2SHWSH,
            },
            "bip48_2" => Purpose::BIP48 {
                script: ScriptType::P2WSH,
            },
            "bip48_3" => Purpose::BIP48 {
                script: ScriptType::P2TR,
            },
            _ => return Err(Error::UnknownPurpose),
        };
        Ok(purpose)
    }
}

impl Serialize for Purpose {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Purpose {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let purpose: String = String::deserialize(deserializer)?;
        Purpose::from_str(&purpose).map_err(serde::de::Error::custom)
    }
}

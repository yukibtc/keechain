// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::fmt;

use uniffi::Error;

pub type Result<T, E = KeechainError> = std::result::Result<T, E>;

#[derive(Error)]
pub enum KeechainError {
    Generic { err: String },
}

impl fmt::Display for KeechainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Generic { err } => write!(f, "{err}"),
        }
    }
}

impl From<keechain_core::bips::bip39::Error> for KeechainError {
    fn from(e: keechain_core::bips::bip39::Error) -> KeechainError {
        Self::Generic { err: e.to_string() }
    }
}

impl From<keechain_core::types::keychain::Error> for KeechainError {
    fn from(e: keechain_core::types::keychain::Error) -> KeechainError {
        Self::Generic { err: e.to_string() }
    }
}

// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use keechain_core::types;
use uniffi::Enum;

pub mod keychain;
pub mod seed;

#[derive(Enum)]
pub enum WordCount {
    W12,
    W18,
    W24,
}

impl From<WordCount> for types::WordCount {
    fn from(value: WordCount) -> Self {
        match value {
            WordCount::W12 => Self::W12,
            WordCount::W18 => Self::W18,
            WordCount::W24 => Self::W24,
        }
    }
}

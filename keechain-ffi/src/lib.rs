// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

mod types;
mod error;

pub use self::types::seed::Seed;
pub use self::error::KeechainError;

uniffi::include_scaffolding!("keechain");
// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_arch = "wasm32")]
pub use instant::SystemTime;

#[cfg(target_arch = "wasm32")]
const UNIX_EPOCH: SystemTime = SystemTime::UNIX_EPOCH;

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

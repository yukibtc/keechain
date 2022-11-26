// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod aes;
pub mod bip;
pub mod convert;
pub mod dir;
pub mod hash;
pub mod io;

pub fn serialize<T>(data: T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    match serde_json::to_string(&data) {
        Ok(data) => Ok(data.into_bytes()),
        Err(_) => Err(anyhow!("Failed to serialize data")),
    }
}

pub fn deserialize<T>(data: Vec<u8>) -> Result<T>
where
    T: DeserializeOwned,
{
    match serde_json::from_slice::<T>(&data) {
        Ok(data) => Ok(data),
        Err(_) => Err(anyhow!("Failed to deserialize data")),
    }
}

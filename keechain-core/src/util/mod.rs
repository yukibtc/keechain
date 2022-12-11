// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use serde::de::DeserializeOwned;
use serde::Serialize;

pub mod bip;
pub mod convert;
pub mod dir;
pub mod format;
pub mod slip;
pub mod time;

use crate::error::{Error, Result};

pub fn serialize<T>(data: T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    match serde_json::to_string(&data) {
        Ok(data) => Ok(data.into_bytes()),
        Err(_) => Err(Error::Generic("Failed to serialize data".to_string())),
    }
}

pub fn deserialize<T>(data: Vec<u8>) -> Result<T>
where
    T: DeserializeOwned,
{
    match serde_json::from_slice::<T>(&data) {
        Ok(data) => Ok(data),
        Err(_) => Err(Error::Generic("Failed to deserialize data".to_string())),
    }
}

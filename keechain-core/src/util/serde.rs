// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Error;

pub fn serialize<T>(data: T) -> Result<Vec<u8>, Error>
where
    T: Serialize,
{
    serde_json::to_vec(&data)
}

pub fn deserialize<T>(data: Vec<u8>) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    serde_json::from_slice::<T>(&data)
}

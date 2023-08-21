// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

pub use base64::DecodeError;

pub fn encode<T>(input: T) -> String
where
    T: AsRef<[u8]>,
{
    base64::encode(input)
}

pub fn decode<T>(input: T) -> Result<Vec<u8>, DecodeError>
where
    T: AsRef<[u8]>,
{
    base64::decode(input)
}

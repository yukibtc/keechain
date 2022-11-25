// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use sha2::{Digest, Sha256};

pub fn sha256<T>(value: T) -> Vec<u8>
where
    T: AsRef<[u8]>,
{
    Sha256::digest(value).to_vec()
}

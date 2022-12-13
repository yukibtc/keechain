// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin::hashes::{sha256, Hash};

pub fn sha256<T>(value: T) -> Vec<u8>
where
    T: AsRef<[u8]>,
{
    sha256::Hash::hash(value.as_ref()).to_vec()
}

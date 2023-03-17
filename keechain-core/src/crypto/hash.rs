// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use bitcoin::hashes::sha256::Hash as Sha256Hash;
use bitcoin::hashes::Hash;

pub fn sha256<T>(value: T) -> Sha256Hash
where
    T: AsRef<[u8]>,
{
    Sha256Hash::hash(value.as_ref())
}

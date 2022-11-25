// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

pub fn bytes_to_hex_string(bytes: Vec<u8>) -> String {
    let mut hash: String = String::new();
    bytes
        .into_iter()
        .for_each(|b| hash.push_str(format!("{:02X}", b).as_str()));
    hash.to_lowercase()
}

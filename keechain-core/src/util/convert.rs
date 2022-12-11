// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

pub fn bytes_to_hex(bytes: Vec<u8>) -> String {
    let mut hash: String = String::new();
    bytes
        .into_iter()
        .for_each(|b| hash.push_str(format!("{:02X}", b).as_str()));
    hash.to_lowercase()
}

pub fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let mut hex_bytes = hex
        .as_bytes()
        .iter()
        .filter_map(|b| match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        })
        .fuse();

    let mut bytes = Vec::new();
    while let (Some(h), Some(l)) = (hex_bytes.next(), hex_bytes.next()) {
        bytes.push(h << 4 | l)
    }
    bytes
}

// Copyright (c) 2021-2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp_nanos() -> u128 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(time) => time.as_nanos(),
        Err(_) => panic!("Invalid system time"),
    }
}

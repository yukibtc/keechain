// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use bdk::bitcoin::hashes::hmac::{Hmac, HmacEngine};
use bdk::bitcoin::hashes::{sha512, Hash, HashEngine};
#[cfg(all(feature = "sysinfo", not(target_vendor = "apple")))]
use bdk::bitcoin::secp256k1::rand;
use bdk::bitcoin::secp256k1::rand::rngs::OsRng;
use bdk::bitcoin::secp256k1::rand::{RngCore, SeedableRng};
pub use bip39::*;
use rand_chacha::ChaCha20Rng;
#[cfg(all(feature = "sysinfo", not(target_vendor = "apple")))]
use sysinfo::{System, SystemExt};

use crate::types::WordCount;
use crate::util::time;

pub fn entropy(word_count: WordCount, custom: Option<Vec<u8>>) -> Vec<u8> {
    let mut h = HmacEngine::<sha512::Hash>::new(b"keechain-entropy");

    // TRNG & CSPRNG
    let mut os_random: [u8; 32] = [0u8; 32];
    OsRng.fill_bytes(&mut os_random);
    h.input(&os_random);

    let mut chacha = ChaCha20Rng::from_entropy();
    let mut chacha_random: [u8; 32] = [0u8; 32];
    chacha.fill_bytes(&mut chacha_random);
    h.input(&chacha_random);

    #[cfg(all(feature = "sysinfo", not(target_vendor = "apple")))]
    if System::IS_SUPPORTED {
        let system_info: System = System::new_all();

        // Dynamic events
        let dynamic_events: Vec<u8> = vec![
            system_info.boot_time().to_be_bytes().to_vec(),
            system_info.total_memory().to_be_bytes().to_vec(),
            system_info.free_memory().to_be_bytes().to_vec(),
            system_info.total_swap().to_be_bytes().to_vec(),
            system_info.free_swap().to_be_bytes().to_vec(),
            format!("{:?}", system_info.processes()).as_bytes().to_vec(),
            format!("{:?}", system_info.load_average())
                .as_bytes()
                .to_vec(),
        ]
        .concat();

        h.input(&dynamic_events);

        // Static events
        let static_events: Vec<u8> = vec![
            system_info
                .host_name()
                .unwrap_or_else(|| rand::random::<u128>().to_string())
                .as_bytes()
                .to_vec(),
            system_info
                .long_os_version()
                .unwrap_or_else(|| rand::random::<u128>().to_string())
                .as_bytes()
                .to_vec(),
            system_info
                .kernel_version()
                .unwrap_or_else(|| rand::random::<u128>().to_string())
                .as_bytes()
                .to_vec(),
            format!("{:?}", system_info.global_cpu_info())
                .as_bytes()
                .to_vec(),
            format!("{:?}", system_info.users()).as_bytes().to_vec(),
        ]
        .concat();

        h.input(&static_events);
    }

    h.input(&time::timestamp_nanos().to_be_bytes());

    // Add custom entropy
    if let Some(custom) = custom {
        h.input(&custom);
    }

    let entropy: [u8; 64] = Hmac::from_engine(h).to_byte_array();
    let len: u32 = word_count.as_u32() * 4 / 3;
    entropy[0..len as usize].to_vec()
}

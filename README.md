# KeeChain

## Description

KeeChain is a Bitcoin application to transform your old **offline** computer in a Signing Device (aka Hardware Wallet).

## Getting started

* [Build from source](doc/build.md) 
* [Usage](doc/usage.md) 

## Features

* Generate BIP39 seed phrase using many sources of entropy:
    - OS source: random bytes provided by OS
    - Dynamic events: timestamp, boot time, total and free memory, total and free SWAP, OS processes and load average
    - Static events: hostname, OS and kernel version, global CPU info and device users
    - Optional: dice roll 🎲
* Restore BIP39 seed phrase with an optional passphrase
* Seed encryption with AES-256
* Export:
    - Descriptors
    - Bitcoin Core descriptors (same as above but already formatted to be inserted into the console using the `importdescriptors` command)
    - Electrum JSON file (BIP44, BIP49 and BIP84)
* Sign and decode PSBT file
* Deterministic Entropy (BIP85)
* Danger:
    - View secrets: entropy, mnemonic, passphrase, HEX seed, BIP32 root key and fingerprint.
    - Wipe: permanently delete keychain

## State

⚠️ **This project is in an ALPHA state, use at YOUR OWN RISK and, possibly, with only testnet coins until release.** ⚠️

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details

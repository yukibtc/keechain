// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use console::Term;
use keechain_core::bips::bip39::Mnemonic;
use keechain_core::bitcoin::psbt::PartiallySignedTransaction;
use keechain_core::bitcoin::secp256k1::Secp256k1;
use keechain_core::bitcoin::Network;
use keechain_core::util::dir;
use keechain_core::{BitcoinCore, Electrum, KeeChain, PsbtUtility, Result, Wasabi};

mod cli;
mod types;
mod util;

use self::cli::io;
use self::cli::{AdvancedCommand, Cli, Command, DangerCommand, ExportTypes, SettingCommand};

fn main() -> Result<()> {
    let args = Cli::parse();
    let secp = Secp256k1::new();
    let network: Network = args.network.into();
    let keychain_path: PathBuf = keechain_common::keychains()?;

    match args.command {
        Command::Generate {
            name,
            word_count,
            dice_roll,
        } => {
            let password: String = io::get_password()?;
            let keechain = KeeChain::generate(
                keychain_path,
                name,
                || Ok(password.clone()),
                io::get_confirmation_password,
                word_count.into(),
                || {
                    if dice_roll {
                        let term = Term::stdout();
                        let mut rolls: Vec<u8> = Vec::new();
                        io::select_dice_roll(term, &mut rolls)?;
                        Ok(Some(rolls))
                    } else {
                        Ok(None)
                    }
                },
                network,
                &secp,
            )?;

            println!("\n!!! WRITE DOWN YOUT SEED PHRASE !!!");
            println!("\n################################################################\n");
            println!("{}", keechain.keychain(password)?.seed.mnemonic());
            println!("\n################################################################\n");

            Ok(())
        }
        Command::Restore { name } => {
            KeeChain::restore(
                keychain_path,
                name,
                io::get_password,
                io::get_confirmation_password,
                || Ok(Mnemonic::from_str(&io::get_input("Seed")?)?),
                network,
                &secp,
            )?;
            Ok(())
        }
        Command::List => {
            let names = dir::get_keychains_list(keychain_path)?;
            for (index, name) in names.iter().enumerate() {
                println!("{}. {name}", index + 1);
            }
            Ok(())
        }
        Command::Identity { name } => {
            let keechain = KeeChain::open(keychain_path, name, io::get_password, network, &secp)?;
            let fingerprint = keechain.identity();
            println!("Fingerprint: {fingerprint}");
            Ok(())
        }
        Command::Export { export_type } => match export_type {
            ExportTypes::Descriptors { name, account } => {
                let password: String = io::get_password()?;
                let keechain =
                    KeeChain::open(keychain_path, name, || Ok(password.clone()), network, &secp)?;
                let descriptors =
                    keechain
                        .keychain(password)?
                        .descriptors(network, Some(account), &secp)?;
                println!("Extenrals:");
                for desc in descriptors.external().iter() {
                    println!("- {desc}");
                }
                println!("Internals:");
                for desc in descriptors.internal().iter() {
                    println!("- {desc}");
                }
                Ok(())
            }
            ExportTypes::BitcoinCore { name, account } => {
                let password: String = io::get_password()?;
                let keechain =
                    KeeChain::open(keychain_path, name, || Ok(password.clone()), network, &secp)?;
                let descriptors =
                    BitcoinCore::new(&keechain.seed(password)?, network, Some(account), &secp)?;
                println!("{}", descriptors.to_string());
                Ok(())
            }
            ExportTypes::Electrum {
                name,
                script,
                account,
            } => {
                let password: String = io::get_password()?;
                let keechain =
                    KeeChain::open(keychain_path, name, || Ok(password.clone()), network, &secp)?;
                let electrum_json_wallet = Electrum::new(
                    &keechain.seed(password)?,
                    network,
                    script.into(),
                    Some(account),
                    &secp,
                )?;
                let path = electrum_json_wallet.save_to_file(keechain_common::home())?;
                println!("Electrum file exported to {}", path.display());
                Ok(())
            }
            ExportTypes::Wasabi { name } => {
                let password: String = io::get_password()?;
                let keechain =
                    KeeChain::open(keychain_path, name, || Ok(password.clone()), network, &secp)?;
                let wasabi_json_wallet = Wasabi::new(&keechain.seed(password)?, network, &secp)?;
                let path = wasabi_json_wallet.save_to_file(keechain_common::home())?;
                println!("Wasabi file exported to {}", path.display());
                Ok(())
            }
        },
        Command::Decode { file, base64 } => {
            let psbt = PartiallySignedTransaction::from_file(file)?;
            if base64 {
                println!("{}", psbt.as_base64());
            } else {
                util::print_psbt(psbt, network);
            }
            Ok(())
        }
        Command::Sign {
            name,
            file,
            descriptor,
        } => {
            let password: String = io::get_password()?;
            let keechain =
                KeeChain::open(keychain_path, name, || Ok(password.clone()), network, &secp)?;
            let seed = &keechain.seed(password)?;
            let mut psbt: PartiallySignedTransaction =
                PartiallySignedTransaction::from_file(&file)?;
            let finalized = match descriptor {
                Some(descriptor) => psbt.sign_with_descriptor(seed, descriptor, network, &secp)?,
                None => psbt.sign_with_seed(seed, network, &secp)?,
            };
            println!("Signed.");
            let mut renamed_file: PathBuf = file;
            dir::rename_psbt(&mut renamed_file, finalized)?;
            psbt.save_to_file(renamed_file)?;
            if finalized {
                println!("PSBT finalized");
            } else {
                println!("PSBT signing not finalized");
            }
            Ok(())
        }
        Command::Advanced { command } => match command {
            AdvancedCommand::Derive {
                name,
                word_count,
                index,
            } => {
                let password: String = io::get_password()?;
                let keechain =
                    KeeChain::open(keychain_path, name, || Ok(password.clone()), network, &secp)?;
                let mnemonic: Mnemonic = keechain.keychain(password)?.deterministic_entropy(
                    word_count.into(),
                    index,
                    &secp,
                )?;
                println!("Mnemonic: {mnemonic}");
                Ok(())
            }
            AdvancedCommand::Danger { command } => match command {
                DangerCommand::ViewSecrets { name } => {
                    let password: String = io::get_password()?;
                    let keechain = KeeChain::open(
                        keychain_path,
                        name,
                        || Ok(password.clone()),
                        network,
                        &secp,
                    )?;
                    let secrets = keechain.keychain(password)?.secrets(network, &secp)?;
                    util::print_secrets(secrets);
                    Ok(())
                }
                DangerCommand::Wipe { name } => {
                    if io::ask("Are you really sure? This action is permanent!")? && io::ask("Again, are you really sure? THIS ACTION IS PERMANENT AND YOU MAY LOSE ALL YOUR FUNDS!")? {
                        let keechain = KeeChain::open(keychain_path, name, io::get_password, network, &secp)?;
                        keechain.wipe()?;
                    } else {
                        println!("Aborted.");
                    }
                    Ok(())
                }
            },
        },
        Command::Setting { command } => match command {
            SettingCommand::Rename { name, new_name } => {
                let mut keechain =
                    KeeChain::open(keychain_path, name, io::get_password, network, &secp)?;
                Ok(keechain.rename(new_name)?)
            }
            SettingCommand::ChangePassword { name } => {
                let mut keechain =
                    KeeChain::open(keychain_path, name, io::get_password, network, &secp)?;
                Ok(keechain.change_password(
                    io::get_password,
                    io::get_new_password,
                    io::get_confirmation_password,
                )?)
            }
        },
    }
}

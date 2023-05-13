// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use keechain_core::bitcoin::psbt::PartiallySignedTransaction;
use keechain_core::bitcoin::{Address, Network, TxOut};
use keechain_core::types::Secrets;
use prettytable::format::FormatBuilder;
use prettytable::{row, Table};

mod format;

pub fn print_secrets(secrets: Secrets) {
    let mut table = Table::new();

    table.add_row(row![
        format!("Entropy ({} bits)", secrets.entropy.len() / 2 * 8),
        secrets.entropy
    ]);
    table.add_row(row!["Mnemonic (BIP39)", secrets.mnemonic]);

    if let Some(passphrase) = &secrets.passphrase {
        table.add_row(row!["Passphrase (BIP39)", passphrase]);
    }

    table.add_row(row!["Seed HEX (BIP39)", secrets.seed_hex]);
    table.add_row(row!["Network", secrets.network]);
    table.add_row(row!["Root Key (BIP32)", secrets.root_key]);
    table.add_row(row!["Fingerprint (BIP32)", secrets.fingerprint]);

    table.printstd();
}

fn output_table_row(network: Network, output: &TxOut) -> String {
    let mut table = Table::new();
    let format = FormatBuilder::new()
        .column_separator('|')
        .padding(0, 0)
        .build();
    table.set_format(format);
    table.add_row(row![
        format!(
            "{} ",
            Address::from_script(&output.script_pubkey, network)
                .expect("Impossible to construct address from output script")
        ),
        format!(" {} sat", format::number(output.value as usize))
    ]);
    table.to_string()
}

pub fn print_psbt(psbt: PartiallySignedTransaction, network: Network) {
    let tx = psbt.extract_tx();
    let inputs_len: usize = tx.input.len();
    let outputs_len: usize = tx.output.len();

    let mut table = Table::new();

    table.set_titles(row![
        format!("Inputs ({inputs_len})"),
        format!("Outputs ({outputs_len})")
    ]);

    if inputs_len >= outputs_len {
        for (index, input) in tx.input.iter().enumerate() {
            let input = format!("{}", input.previous_output);
            if let Some(output) = tx.output.get(index) {
                table.add_row(row![input, output_table_row(network, output)]);
            } else {
                table.add_row(row![input, ""]);
            }
        }
    } else {
        for (index, output) in tx.output.iter().enumerate() {
            let output = output_table_row(network, output);
            if let Some(input) = tx.input.get(index) {
                table.add_row(row![format!("{}", input.previous_output), output]);
            } else {
                table.add_row(row!["", output]);
            }
        }
    }

    table.printstd();
}

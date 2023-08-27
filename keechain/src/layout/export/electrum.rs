// Copyright (c) 2022-2023 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;
use std::str::FromStr;

use eframe::egui::{Align, ComboBox, Layout, RichText, Ui};
use keechain_core::bitcoin::Network;
use keechain_core::types::{Electrum, ElectrumSupportedScripts, Index, Seed};
use keechain_core::Result;

use crate::component::{Button, Error, Heading, Identity, InputField, View};
use crate::theme::color::{DARK_GREEN, ORANGE};
use crate::{AppState, Menu, Stage, SECP256K1};

fn export_electrum(
    seed: Seed,
    network: Network,
    script: ElectrumSupportedScripts,
    account: Option<u32>,
) -> Result<PathBuf> {
    let electrum_json_wallet = Electrum::new(seed, network, script, account, &SECP256K1)?;
    let home_dir: PathBuf = keechain_common::home();
    Ok(electrum_json_wallet.save_to_file(home_dir)?)
}

const WALLET_TYPES: [(ElectrumSupportedScripts, &str); 3] = [
    (ElectrumSupportedScripts::Legacy, "Legacy (BIP44)"),
    (ElectrumSupportedScripts::Segwit, "Segwit (BIP49)"),
    (
        ElectrumSupportedScripts::NativeSegwit,
        "Native Segwit (BIP84)",
    ),
];

#[derive(Default)]
pub struct ExportElectrumState {
    script: ElectrumSupportedScripts,
    account: String,
    result: Option<String>,
    error: Option<String>,
}

impl ExportElectrumState {
    pub fn clear(&mut self) {
        self.script = ElectrumSupportedScripts::default();
        self.account = String::new();
        self.result = None;
        self.error = None;
    }
}

pub fn update(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    View::show(ui, |ui| {
        Heading::new("Export Electrum").render(ui);

        if let Some(keechain) = &app.keechain {
            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            ui.add_space(15.0);
        }

        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            ui.add_space(1.0);
            ui.label("Type");
            ui.horizontal_wrapped(|ui| {
                ComboBox::from_id_source("type")
                    .width(ui.available_width())
                    .selected_text(
                        WALLET_TYPES
                            .iter()
                            .find(|&&t| t.0 == app.layouts.export_electrum.script)
                            .map(|t| t.1)
                            .unwrap_or("Impossible to get value"),
                    )
                    .show_ui(ui, |ui| {
                        for (script, label) in WALLET_TYPES.into_iter() {
                            ui.selectable_value(
                                &mut app.layouts.export_electrum.script,
                                script,
                                label,
                            );
                        }
                    });
            })
        });

        ui.add_space(7.0);

        InputField::new("Account")
            .placeholder("Account (between 0 and 2^31 - 1)")
            .render(ui, &mut app.layouts.export_electrum.account);

        if let Some(result) = &app.layouts.export_electrum.result {
            ui.add_space(7.0);
            ui.label(RichText::new(result).color(DARK_GREEN));
        }

        if let Some(error) = &app.layouts.export_electrum.error {
            ui.add_space(7.0);
            Error::new(error).render(ui);
        }

        ui.add_space(15.0);

        let is_ready: bool = !app.layouts.export_electrum.account.is_empty();

        let button = Button::new("Export")
            .background_color(ORANGE)
            .enabled(is_ready)
            .render(ui);

        if is_ready && button.clicked() {
            match app.keechain.as_mut() {
                Some(keechain) => {
                    match Index::from_str(app.layouts.export_electrum.account.as_str()) {
                        Ok(index) => {
                            match export_electrum(
                                keechain.keychain.seed(),
                                app.network,
                                app.layouts.export_electrum.script,
                                Some(index.as_u32()),
                            ) {
                                Ok(path) => {
                                    app.layouts.export_electrum.error = None;
                                    app.layouts.export_electrum.result =
                                        Some(format!("File exported to {}", path.display()));
                                }
                                Err(e) => app.layouts.export_electrum.error = Some(e.to_string()),
                            }
                        }
                        Err(e) => app.layouts.export_electrum.error = Some(e.to_string()),
                    }
                }
                None => {
                    app.layouts.export_electrum.error =
                        Some("Impossible to get keechain".to_string())
                }
            }
        }

        ui.add_space(5.0);

        if Button::new("Back").render(ui).clicked() {
            app.layouts.export_electrum.clear();
            app.stage = Stage::Menu(Menu::Export);
        }
    });
}

// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::{Path, PathBuf};
use std::str::FromStr;

use eframe::egui::{RichText, Ui};
use keechain_core::bdk::miniscript::Descriptor;
use keechain_core::bitcoin::psbt::PartiallySignedTransaction;
use keechain_core::bitcoin::Network;
use keechain_core::types::{Psbt, Seed};
use keechain_core::util::dir;
use rfd::FileDialog;

use crate::component::{Button, Error, Heading, Identity, InputField, View};
use crate::theme::color::{DARK_GREEN, DARK_RED, ORANGE};
use crate::{AppState, Menu, Stage};

pub fn sign_file_from_seed<P>(
    seed: &Seed,
    descriptor: String,
    network: Network,
    path: P,
) -> crate::Result<bool>
where
    P: AsRef<Path>,
{
    let psbt_file = path.as_ref();
    let mut psbt: PartiallySignedTransaction = PartiallySignedTransaction::from_file(psbt_file)?;
    let finalized: bool = if descriptor.is_empty() {
        psbt.sign(seed, network)?
    } else {
        let descriptor = Descriptor::from_str(&descriptor)?;
        psbt.sign_with_descriptor(seed, descriptor, false, network)?
    };
    let mut psbt_file: PathBuf = psbt_file.to_path_buf();
    dir::rename_psbt(&mut psbt_file, finalized)?;
    psbt.save_to_file(psbt_file)?;
    Ok(finalized)
}

#[allow(dead_code)]
pub struct PsbtFile {
    psbt: PartiallySignedTransaction,
    path: PathBuf,
}

#[derive(Default)]
pub struct SignState {
    descriptor: String,
    custom_descriptor: bool,
    psbt_file: Option<PsbtFile>,
    error: Option<String>,
    finish: bool,
}

impl SignState {
    pub fn clear(&mut self) {
        self.descriptor = String::new();
        self.custom_descriptor = false;
        self.psbt_file = None;
        self.error = None;
        self.finish = false;
    }
}

pub fn update(app: &mut AppState, ui: &mut Ui) {
    if let Some(keechain) = &app.keechain {
        View::show(ui, |ui| {
            Heading::new("Sign").render(ui);

            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            ui.add_space(15.0);

            if let Some(error) = &app.layouts.sign.error {
                Error::new(error).render(ui);
                ui.add_space(7.0);
            }

            let is_signed: bool = app.layouts.sign.finish;
            let is_ready_to_sign: bool = app.layouts.sign.psbt_file.is_some();

            if !is_ready_to_sign && !is_signed {
                let button = Button::new("Select PSBT file")
                    .background_color(DARK_GREEN)
                    .render(ui);

                if button.clicked() {
                    if let Some(path) = FileDialog::new().add_filter("psbt", &["psbt"]).pick_file()
                    {
                        match Psbt::from_file(path.clone()) {
                            Ok(psbt) => {
                                app.layouts.sign.error = None;
                                app.layouts.sign.psbt_file = Some(PsbtFile { psbt, path });
                            }
                            Err(e) => app.layouts.sign.error = Some(e.to_string()),
                        }
                    }
                }
            }

            if is_ready_to_sign && !is_signed {
                if app.layouts.sign.custom_descriptor {
                    InputField::new("Custom descriptor (optional)")
                        .placeholder("Custom descriptor (ex. multisig desc")
                        .rows(3)
                        .render(ui, &mut app.layouts.sign.descriptor);
                }
                ui.checkbox(
                    &mut app.layouts.sign.custom_descriptor,
                    "Use custom descriptor",
                );
                if let Some(psbt_file) = app.layouts.sign.psbt_file.as_ref() {
                    if Button::new("Sign")
                        .background_color(ORANGE)
                        .render(ui)
                        .clicked()
                    {
                        match sign_file_from_seed(
                            &keechain.keychain.seed(),
                            app.layouts.sign.descriptor.clone(),
                            app.network,
                            psbt_file.path.clone(),
                        ) {
                            Ok(finalized) => {
                                app.layouts.sign.clear();
                                app.layouts.sign.finish = true;
                                if !finalized {
                                    app.layouts.sign.error =
                                        Some("PSBT signed but not finalized".to_string());
                                }
                            }
                            Err(e) => app.layouts.sign.error = Some(e.to_string()),
                        }
                    }
                }

                ui.add_space(5.0);

                if Button::new("Clear")
                    .background_color(DARK_RED)
                    .render(ui)
                    .clicked()
                {
                    app.layouts.sign.clear();
                }
            }

            if is_signed {
                ui.label(RichText::new("PSBT signed!").color(DARK_GREEN));
                ui.add_space(5.0);
                if Button::new("Sign again").render(ui).clicked() {
                    app.layouts.sign.clear();
                }
            }

            ui.add_space(5.0);

            if Button::new("Back").render(ui).clicked() {
                app.layouts.sign.clear();
                app.stage = Stage::Menu(Menu::Main);
            }
        });
    } else {
        app.set_stage(Stage::Start);
    }
}

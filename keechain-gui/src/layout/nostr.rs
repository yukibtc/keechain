// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use copypasta::{ClipboardContext, ClipboardProvider};
use eframe::egui::{Align, Layout, RichText, ScrollArea, Ui};
use keechain_core::bitcoin::secp256k1::schnorr::Signature;
use keechain_core::bitcoin::secp256k1::{Secp256k1, SecretKey};
use keechain_core::bitcoin::XOnlyPublicKey;
use keechain_core::nostr::{nip06, nip26};
use keechain_core::types::Seed;
use keechain_core::util::bip::bip32::Bip32RootKey;

use crate::component::{Button, Heading, InputField, Version};
use crate::theme::color::{DARK_RED, ORANGE, RED};
use crate::{AppState, Menu, Stage};

#[derive(Clone, Default)]
pub struct NostrState {
    secret_key: Option<SecretKey>,
    delegatee_pk: String,
    conditions: String,
    signature: Option<Signature>,
    error: Option<String>,
}

impl NostrState {
    pub fn clear(&mut self) {
        self.secret_key = None;
        self.delegatee_pk = String::new();
        self.conditions = String::new();
        self.signature = None;
        self.error = None;
    }
}

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    if app.keechain.is_none() {
        app.set_stage(Stage::Start);
    }

    ScrollArea::vertical().show(ui, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.set_max_width(ui.available_width() - 20.0);

            Heading::new("Nostr").render(ui);

            ui.add_space(15.0);

            if let Some(keechain) = &app.keechain {
                let seed: Seed = keechain.keychain.seed();
                ui.group(|ui| {
                    if let Ok(fingerprint) = seed.fingerprint(app.network) {
                        ui.label(format!("Fingerprint: {}", fingerprint));
                    }
                    ui.label(format!("Using passphrase: {}", seed.passphrase().is_some()));
                });
                if let Ok(secret_key) = nip06::derive_secret_key_from_seed(seed) {
                    app.layouts.nostr.secret_key = Some(secret_key);
                }
            }

            ui.add_space(15.0);

            if let Some(secret_key) = &app.layouts.nostr.secret_key {
                let secp = Secp256k1::new();
                let secret_key_str = secret_key.display_secret().to_string();
                let public_key_str = secret_key.public_key(&secp).to_string();
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Secret key: {}..{}",
                        &secret_key_str[0..8],
                        &secret_key_str[secret_key_str.len() - 8..]
                    ));
                    ui.add_space(7.0);
                    if ui.button("📋").clicked() {
                        match ClipboardContext::new() {
                            Ok(mut ctx) => {
                                if let Err(e) = ctx.set_contents(secret_key_str) {
                                    app.layouts.nostr.error = Some(e.to_string());
                                }
                            }
                            Err(e) => app.layouts.nostr.error = Some(e.to_string()),
                        }
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Public key: {}..{}",
                        &public_key_str[0..8],
                        &public_key_str[public_key_str.len() - 8..]
                    ));
                    ui.add_space(7.0);
                    if ui.button("📋").clicked() {
                        match ClipboardContext::new() {
                            Ok(mut ctx) => {
                                if let Err(e) = ctx.set_contents(public_key_str) {
                                    app.layouts.nostr.error = Some(e.to_string());
                                }
                            }
                            Err(e) => app.layouts.nostr.error = Some(e.to_string()),
                        }
                    }
                });
            }

            ui.add_space(7.0);
            ui.separator();
            ui.add_space(7.0);

            Heading::new("Sign delegation").size(22.0).render(ui);

            ui.add_space(15.0);

            if app.layouts.nostr.signature.is_none() {
                InputField::new("Delegatee public key")
                    .placeholder("Delegatee public key")
                    .render(ui, &mut app.layouts.nostr.delegatee_pk);

                ui.add_space(7.0);

                InputField::new("Conditions")
                    .placeholder("Conditions")
                    .render(ui, &mut app.layouts.nostr.conditions);
            }

            if let Some(signature) = app.layouts.nostr.signature {
                ui.add_space(7.0);
                ui.label(format!("Signature: {}", signature));
            }

            ui.add_space(7.0);

            if let Some(error) = &app.layouts.nostr.error {
                ui.label(RichText::new(error).color(RED));
            }

            ui.add_space(15.0);

            let is_ready: bool = app.layouts.nostr.secret_key.is_some()
                && !app.layouts.nostr.delegatee_pk.is_empty()
                && !app.layouts.nostr.conditions.is_empty();

            if let Some(signature) = &app.layouts.nostr.signature {
                if Button::new("Copy")
                    .background_color(ORANGE)
                    .render(ui)
                    .clicked()
                {
                    match ClipboardContext::new() {
                        Ok(mut ctx) => {
                            if let Err(e) = ctx.set_contents(signature.to_string()) {
                                app.layouts.nostr.error = Some(e.to_string());
                            }
                        }
                        Err(e) => app.layouts.nostr.error = Some(e.to_string()),
                    }
                }
                ui.add_space(5.0);
                if Button::new("Clear")
                    .background_color(DARK_RED)
                    .render(ui)
                    .clicked()
                {
                    app.layouts.nostr.clear();
                }
            } else {
                let button = Button::new("Sign")
                    .enabled(is_ready)
                    .background_color(ORANGE)
                    .render(ui);

                if is_ready && button.clicked() {
                    if let Some(secret_key) = &app.layouts.nostr.secret_key {
                        match XOnlyPublicKey::from_str(&app.layouts.nostr.delegatee_pk) {
                            Ok(delegatee_pk) => {
                                match nip26::sign_delegation(
                                    secret_key,
                                    delegatee_pk,
                                    app.layouts.nostr.conditions.clone(),
                                ) {
                                    Ok(sig) => {
                                        app.layouts.nostr.error = None;
                                        app.layouts.nostr.signature = Some(sig);
                                    }
                                    Err(e) => app.layouts.nostr.error = Some(e.to_string()),
                                }
                            }
                            Err(e) => app.layouts.nostr.error = Some(e.to_string()),
                        }
                    }
                }
            }

            ui.add_space(5.0);

            if Button::new("Back").render(ui).clicked() {
                app.layouts.nostr.clear();
                app.stage = Stage::Menu(Menu::Other);
            }
        });

        ui.add_space(20.0);

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            Version::new().render(ui)
        });
    });
}

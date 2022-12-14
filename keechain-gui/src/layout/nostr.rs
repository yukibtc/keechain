// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use eframe::egui::{Align, Layout, RichText, Ui};
use keechain_core::bitcoin::secp256k1::schnorr::Signature;
use keechain_core::bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use keechain_core::bitcoin::XOnlyPublicKey;
use keechain_core::nostr::{nip06, nip26, ToBech32};
use keechain_core::types::Seed;

use crate::component::{Button, Heading, Identity, InputField, ReadOnlyField, View};
use crate::theme::color::{DARK_RED, ORANGE, RED};
use crate::{AppState, Menu, Stage};

pub struct Keys {
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl Keys {
    pub fn new(secret_key: SecretKey) -> Self {
        let secp = Secp256k1::new();
        Self {
            public_key: secret_key.public_key(&secp),
            secret_key,
        }
    }
}

#[derive(Default)]
pub struct NostrState {
    keys: Option<Keys>,
    bech32: bool,
    delegatee_pk: String,
    conditions: String,
    signature: Option<Signature>,
    error: Option<String>,
}

impl NostrState {
    pub fn clear(&mut self) {
        self.keys = None;
        self.bech32 = false;
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

    View::show(ui, |ui| {
        Heading::new("Nostr").render(ui);

        if let Some(keechain) = &app.keechain {
            let seed: Seed = keechain.keychain.seed();
            Identity::new(keechain.keychain.seed(), app.network).render(ui);
            if let Ok(secret_key) = nip06::derive_secret_key_from_seed(seed) {
                app.layouts.nostr.keys = Some(Keys::new(secret_key));
            }
        }

        ui.add_space(7.0);

        if let Some(keys) = &app.layouts.nostr.keys {
            if app.layouts.nostr.bech32 {
                ReadOnlyField::new(
                    "Secret key",
                    keys.secret_key.to_bech32().expect("Impossible to convert"),
                )
                .render(ui);
                ReadOnlyField::new(
                    "Public key",
                    keys.public_key.to_bech32().expect("Impossible to convert"),
                )
                .render(ui);
            } else {
                ReadOnlyField::new("Secret key", keys.secret_key.display_secret().to_string())
                    .render(ui);
                ReadOnlyField::new("Public key", keys.public_key.to_string()).render(ui);
            }
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.checkbox(&mut app.layouts.nostr.bech32, "Bech32 format");
            });
        }

        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);

        Heading::new("Sign delegation").size(22.0).render(ui);

        if let Some(signature) = app.layouts.nostr.signature {
            ReadOnlyField::new("Signature", signature.to_string())
                .rows(5)
                .render(ui);

            ui.add_space(15.0);

            if Button::new("Clear")
                .background_color(DARK_RED)
                .render(ui)
                .clicked()
            {
                app.layouts.nostr.clear();
            }
        } else {
            InputField::new("Delegatee public key")
                .placeholder("Delegatee public key")
                .render(ui, &mut app.layouts.nostr.delegatee_pk);

            ui.add_space(7.0);

            InputField::new("Conditions")
                .placeholder("Conditions")
                .render(ui, &mut app.layouts.nostr.conditions);

            if let Some(error) = &app.layouts.nostr.error {
                ui.add_space(7.0);
                ui.label(RichText::new(error).color(RED));
            }

            ui.add_space(15.0);

            let is_ready: bool = app.layouts.nostr.keys.is_some()
                && !app.layouts.nostr.delegatee_pk.is_empty()
                && !app.layouts.nostr.conditions.is_empty();

            let button = Button::new("Sign")
                .enabled(is_ready)
                .background_color(ORANGE)
                .render(ui);

            if is_ready && button.clicked() {
                if let Some(keys) = &app.layouts.nostr.keys {
                    match XOnlyPublicKey::from_str(&app.layouts.nostr.delegatee_pk) {
                        Ok(delegatee_pk) => {
                            match nip26::sign_delegation(
                                &keys.secret_key,
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
}

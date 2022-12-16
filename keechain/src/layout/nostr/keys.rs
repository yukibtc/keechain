// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, Layout, Ui};
use keechain_core::bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use keechain_core::nostr::{nip06, ToBech32};
use keechain_core::types::Seed;

use crate::component::{Button, Heading, Identity, ReadOnlyField, View};
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
pub struct NostrKeysState {
    keys: Option<Keys>,
    bech32: bool,
    error: Option<String>,
}

impl NostrKeysState {
    pub fn clear(&mut self) {
        self.keys = None;
        self.bech32 = false;
        self.error = None;
    }
}

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    if let Some(keechain) = &app.keechain {
        View::show(ui, |ui| {
            Heading::new("Nostr").render(ui);

            let seed: Seed = keechain.keychain.seed();
            Identity::new(keechain.keychain.seed(), app.network).render(ui);

            ui.add_space(7.0);

            if let Some(keys) = &app.layouts.nostr_keys.keys {
                if app.layouts.nostr_keys.bech32 {
                    ReadOnlyField::new(
                        "Secret key",
                        keys.secret_key.to_bech32().expect("Impossible to convert"),
                    )
                    .rows(3)
                    .render(ui);
                    ReadOnlyField::new(
                        "Public key",
                        keys.public_key.to_bech32().expect("Impossible to convert"),
                    )
                    .rows(3)
                    .render(ui);
                } else {
                    ReadOnlyField::new("Secret key", keys.secret_key.display_secret().to_string())
                        .rows(3)
                        .render(ui);
                    ReadOnlyField::new("Public key", keys.public_key.to_string())
                        .rows(3)
                        .render(ui);
                }
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.checkbox(&mut app.layouts.nostr_keys.bech32, "Bech32 format");
                });
            } else if let Ok(secret_key) = nip06::derive_secret_key_from_seed(seed) {
                app.layouts.nostr_keys.keys = Some(Keys::new(secret_key));
            }

            ui.add_space(5.0);

            if Button::new("Back").render(ui).clicked() {
                app.layouts.nostr_keys.clear();
                app.stage = Stage::Menu(Menu::Nostr);
            }
        });
    } else {
        app.set_stage(Stage::Start);
    }
}

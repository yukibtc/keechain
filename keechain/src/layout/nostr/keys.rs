// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use eframe::egui::{Align, Layout, Ui};
use keechain_core::bitcoin::secp256k1::SecretKey;
use keechain_core::bitcoin::XOnlyPublicKey;
use keechain_core::nostr::util::nips::nip06::FromMnemonic;
use keechain_core::nostr::util::nips::nip19::ToBech32;
use keechain_core::nostr::Keys;
use keechain_core::types::Seed;

use crate::component::{Button, Heading, Identity, ReadOnlyField, View};
use crate::{AppState, Menu, Stage};

#[derive(Default)]
pub struct NostrKeysState {
    keys: Option<(SecretKey, XOnlyPublicKey)>,
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

pub fn update(app: &mut AppState, ui: &mut Ui) {
    if let Some(keechain) = &app.keechain {
        View::show(ui, |ui| {
            Heading::new("Nostr").render(ui);

            let seed: Seed = keechain.keychain.seed();
            Identity::new(keechain.keychain.seed(), app.network).render(ui);

            ui.add_space(15.0);

            if let Some(keys) = &app.layouts.nostr_keys.keys {
                if app.layouts.nostr_keys.bech32 {
                    ReadOnlyField::new(
                        "Secret key",
                        keys.0.to_bech32().expect("Impossible to convert"),
                    )
                    .rows(3)
                    .render(ui);
                    ReadOnlyField::new(
                        "Public key",
                        keys.1.to_bech32().expect("Impossible to convert"),
                    )
                    .rows(3)
                    .render(ui);
                } else {
                    ReadOnlyField::new("Secret key", keys.0.display_secret().to_string())
                        .rows(3)
                        .render(ui);
                    ReadOnlyField::new("Public key", keys.1.to_string())
                        .rows(3)
                        .render(ui);
                }
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.checkbox(&mut app.layouts.nostr_keys.bech32, "Bech32 format");
                });
            } else if let Ok(keys) =
                Keys::from_mnemonic(seed.mnemonic().to_string(), seed.passphrase())
            {
                match keys.secret_key() {
                    Ok(secret_key) => {
                        app.layouts.nostr_keys.keys = Some((secret_key, keys.public_key()))
                    }
                    Err(e) => app.layouts.nostr_keys.error = Some(e.to_string()),
                }
            }

            ui.add_space(15.0);

            if Button::new("Back").render(ui).clicked() {
                app.layouts.nostr_keys.clear();
                app.stage = Stage::Menu(Menu::Nostr);
            }
        });
    } else {
        app.set_stage(Stage::Start);
    }
}

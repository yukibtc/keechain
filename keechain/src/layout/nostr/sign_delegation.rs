// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::str::FromStr;

use eframe::egui::Ui;
use keechain_core::bitcoin::secp256k1::schnorr::Signature;
use keechain_core::bitcoin::XOnlyPublicKey;
use keechain_core::nostr::{nip06, nip26};
use keechain_core::types::Seed;

use crate::component::{Button, Error, Heading, Identity, InputField, ReadOnlyField, View};
use crate::theme::color::{DARK_RED, ORANGE};
use crate::{AppState, Menu, Stage};

#[derive(Default)]
pub struct NostrSignDelegationState {
    delegatee_pk: String,
    conditions: String,
    signature: Option<Signature>,
    error: Option<String>,
}

impl NostrSignDelegationState {
    pub fn clear(&mut self) {
        self.delegatee_pk = String::new();
        self.conditions = String::new();
        self.signature = None;
        self.error = None;
    }
}

pub fn update_layout(app: &mut AppState, ui: &mut Ui) {
    if let Some(keechain) = &app.keechain {
        View::show(ui, |ui| {
            Heading::new("Sign delegation").render(ui);

            let seed: Seed = keechain.keychain.seed();
            Identity::new(keechain.keychain.seed(), app.network).render(ui);

            ui.add_space(15.0);

            InputField::new("Delegatee public key")
                .placeholder("Delegatee public key")
                .render(ui, &mut app.layouts.nostr_sign_delegation.delegatee_pk);

            ui.add_space(7.0);

            InputField::new("Conditions")
                .placeholder("Conditions")
                .render(ui, &mut app.layouts.nostr_sign_delegation.conditions);

            if let Some(signature) = app.layouts.nostr_sign_delegation.signature {
                ui.add_space(7.0);
                ReadOnlyField::new("Signature", signature.to_string())
                    .rows(5)
                    .render(ui);
            }

            if let Some(error) = &app.layouts.nostr_sign_delegation.error {
                ui.add_space(7.0);
                Error::new(error).render(ui);
            }

            ui.add_space(15.0);

            let is_ready: bool = !app.layouts.nostr_sign_delegation.delegatee_pk.is_empty()
                && !app.layouts.nostr_sign_delegation.conditions.is_empty();

            let button = Button::new("Sign")
                .enabled(is_ready)
                .background_color(ORANGE)
                .render(ui);

            ui.add_space(5.0);

            if Button::new("Clear")
                .background_color(DARK_RED)
                .enabled(app.layouts.nostr_sign_delegation.signature.is_some())
                .render(ui)
                .clicked()
            {
                app.layouts.nostr_sign_delegation.clear();
            }

            if is_ready && button.clicked() {
                match nip06::derive_secret_key_from_seed(seed) {
                    Ok(secret_key) => match XOnlyPublicKey::from_str(
                        &app.layouts.nostr_sign_delegation.delegatee_pk,
                    ) {
                        Ok(delegatee_pk) => {
                            match nip26::sign_delegation(
                                &secret_key,
                                delegatee_pk,
                                app.layouts.nostr_sign_delegation.conditions.clone(),
                            ) {
                                Ok(sig) => {
                                    app.layouts.nostr_sign_delegation.error = None;
                                    app.layouts.nostr_sign_delegation.signature = Some(sig);
                                }
                                Err(e) => {
                                    app.layouts.nostr_sign_delegation.error = Some(e.to_string())
                                }
                            }
                        }
                        Err(e) => app.layouts.nostr_sign_delegation.error = Some(e.to_string()),
                    },
                    Err(e) => app.layouts.nostr_sign_delegation.error = Some(e.to_string()),
                }
            }

            ui.add_space(5.0);

            if Button::new("Back").render(ui).clicked() {
                app.layouts.nostr_sign_delegation.clear();
                app.stage = Stage::Menu(Menu::Nostr);
            }
        });
    } else {
        app.set_stage(Stage::Start);
    }
}

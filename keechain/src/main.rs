// Copyright (c) 2022 Yuki Kishimoto
// Distributed under the MIT software license

use std::path::PathBuf;
use std::str::FromStr;

use eframe::egui::{self, CentralPanel, Context};
use eframe::epaint::FontFamily::Proportional;
use eframe::epaint::{FontId, Vec2};
use eframe::{App, Frame, NativeOptions, Theme};
use egui::TextStyle::{Body, Button, Heading, Monospace, Small};
use keechain_core::bitcoin::Network;
use keechain_core::types::KeeChain;
use keechain_core::Result;
use once_cell::sync::Lazy;

mod component;
mod layout;
mod theme;

use self::layout::{
    ChangePasswordState, DeterministicEntropyState, ExportElectrumState, NewKeychainState,
    PassphraseState, RenameKeychainState, RestoreState, SignState, StartState, ViewSecretsState,
    WipeKeychainState,
};
#[cfg(feature = "nostr")]
use self::layout::{NostrKeysState, NostrSignDelegationState};

const MIN_WINDOWS_SIZE: Vec2 = egui::vec2(350.0, 530.0);
const GENERIC_FONT_HEIGHT: f32 = 18.0;

static KEYCHAINS_PATH: Lazy<PathBuf> =
    Lazy::new(|| keechain_common::keychains().expect("Can't get keychains path"));

fn parse_network(args: Vec<String>) -> Result<Network> {
    for (i, arg) in args.iter().enumerate() {
        if arg.contains("--") {
            let network = Network::from_str(args[i].trim_start_matches("--"))?;
            return Ok(network);
        }
    }
    Ok(Network::Bitcoin)
}

pub fn main() -> Result<()> {
    let network: Network = parse_network(std::env::args().collect())?;
    let options = NativeOptions {
        fullscreen: false,
        resizable: true,
        always_on_top: false,
        default_theme: Theme::Dark,
        follow_system_theme: false,
        initial_window_size: Some(MIN_WINDOWS_SIZE),
        min_window_size: Some(MIN_WINDOWS_SIZE),
        drag_and_drop_support: false,
        ..Default::default()
    };
    let app = AppState::new(&network);
    let app_name = format!(
        "KeeChain{}",
        if network.ne(&Network::Bitcoin) {
            format!(" [{network}]")
        } else {
            String::new()
        }
    );
    eframe::run_native(&app_name, options, Box::new(|_cc| Box::new(app)));
    Ok(())
}

#[derive(Clone)]
pub enum ExportTypes {
    Descriptors,
    BitcoinCore,
    Electrum,
}

pub enum Command {
    Passphrase,
    Sign,
    Export(ExportTypes),
    #[cfg(feature = "nostr")]
    NostrKeys,
    #[cfg(feature = "nostr")]
    NostrSignDelegation,
    RenameKeychain,
    ChangePassword,
    ViewSecrets,
    WipeKeychain,
    DeterministicEntropy,
}

#[derive(Clone)]
pub enum Menu {
    Main,
    Export,
    Advanced,
    Setting,
    Danger,
    #[cfg(feature = "nostr")]
    Nostr,
}

pub enum Stage {
    Start,
    NewKeychain,
    RestoreKeychain,
    Menu(Menu),
    Command(Command),
}

impl Default for Stage {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Default)]
pub struct AppLayoutStates {
    start: StartState,
    new_keychain: NewKeychainState,
    restore: RestoreState,
    sign: SignState,
    passphrase: PassphraseState,
    #[cfg(feature = "nostr")]
    nostr_keys: NostrKeysState,
    #[cfg(feature = "nostr")]
    nostr_sign_delegation: NostrSignDelegationState,
    rename_keychain: RenameKeychainState,
    change_password: ChangePasswordState,
    view_secrets: ViewSecretsState,
    wipe_keychain: WipeKeychainState,
    deterministic_entropy: DeterministicEntropyState,
    export_electrum: ExportElectrumState,
}

pub struct AppState {
    network: Network,
    stage: Stage,
    keechain: Option<KeeChain>,
    layouts: AppLayoutStates,
}

impl AppState {
    pub fn new(network: &Network) -> Self {
        Self {
            network: *network,
            stage: Stage::default(),
            keechain: None,
            layouts: AppLayoutStates::default(),
        }
    }

    fn set_stage(&mut self, stage: Stage) {
        self.stage = stage;
    }

    fn set_keechain(&mut self, keechain: Option<KeeChain>) {
        self.keechain = keechain;
    }
}

impl App for AppState {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (Heading, FontId::new(28.0, Proportional)),
            (Body, FontId::new(GENERIC_FONT_HEIGHT, Proportional)),
            (Monospace, FontId::new(GENERIC_FONT_HEIGHT, Proportional)),
            (Button, FontId::new(GENERIC_FONT_HEIGHT, Proportional)),
            (Small, FontId::new(16.0, Proportional)),
        ]
        .into();
        ctx.set_style(style);

        CentralPanel::default().show(ctx, |ui| match &self.stage {
            Stage::Start => layout::start::update(self, ui),
            Stage::NewKeychain => layout::new_keychain::update(self, ui),
            Stage::RestoreKeychain => layout::restore::update(self, ui),
            Stage::Menu(menu) => layout::menu::update(self, menu.clone(), ui, frame),
            Stage::Command(cmd) => match cmd {
                Command::Passphrase => layout::passphrase::update(self, ui),
                Command::Sign => layout::sign::update(self, ui),
                Command::Export(export_type) => {
                    layout::export::update(self, export_type.clone(), ui)
                }
                #[cfg(feature = "nostr")]
                Command::NostrKeys => layout::nostr::keys::update(self, ui),
                #[cfg(feature = "nostr")]
                Command::NostrSignDelegation => layout::nostr::sign_delegation::update(self, ui),
                Command::RenameKeychain => layout::setting::rename::update(self, ui),
                Command::ChangePassword => layout::setting::change_password::update(self, ui),
                Command::ViewSecrets => layout::advanced::danger::view_secrets::update(self, ui),
                Command::WipeKeychain => layout::advanced::danger::wipe::update(self, ui),
                Command::DeterministicEntropy => {
                    layout::advanced::deterministic_entropy::update(self, ui)
                }
            },
        });
    }
}

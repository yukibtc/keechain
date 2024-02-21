// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

#![forbid(unsafe_code)]
#![windows_subsystem = "windows"]

use std::path::PathBuf;
use std::str::FromStr;

use iced::{executor, font, Application, Command, Element, Pixels, Settings, Theme};
use keechain_core::bitcoin::secp256k1::{rand, All, Secp256k1};
use keechain_core::bitcoin::Network;
use once_cell::sync::Lazy;

mod app;
mod component;
mod constants;
mod start;
mod theme;

use self::constants::{APP_NAME, DEFAULT_FONT_SIZE};
use self::theme::font::{
    REGULAR, ROBOTO_MONO_BOLD_BYTES, ROBOTO_MONO_LIGHT_BYTES, ROBOTO_MONO_REGULAR_BYTES,
};

static SECP256K1: Lazy<Secp256k1<All>> = Lazy::new(|| {
    let mut ctx = Secp256k1::new();
    let mut rng = rand::thread_rng();
    ctx.randomize(&mut rng);
    ctx
});
static BASE_PATH: Lazy<PathBuf> =
    Lazy::new(|| keechain_common::keychains().expect("Can't get keychains path"));

fn parse_network(args: Vec<String>) -> Network {
    for (i, arg) in args.iter().enumerate() {
        if arg.contains("--") {
            let network = Network::from_str(args[i].trim_start_matches("--")).unwrap();
            return network;
        }
    }
    Network::Bitcoin
}

pub fn main() -> iced::Result {
    let network: Network = parse_network(std::env::args().collect());
    let mut settings = Settings::with_flags(network);
    settings.id = Some(String::from("org.keechain.desktop"));
    settings.window.min_size = Some((350, 560));
    settings.window.size = (350, 560);
    settings.antialiasing = false;
    settings.default_text_size = Pixels::from(DEFAULT_FONT_SIZE as f32);
    settings.default_font = REGULAR;

    KeechainApp::run(settings)
}

pub struct KeechainApp {
    state: State,
}
pub enum State {
    Start(start::Start),
    App(Box<app::App>),
}

#[derive(Debug, Clone)]
pub enum Message {
    Start(Box<start::Message>),
    App(Box<app::Message>),
    FontLoaded(Result<(), font::Error>),
}

impl Application for KeechainApp {
    type Executor = executor::Default;
    type Flags = Network;
    type Message = Message;
    type Theme = Theme;

    fn new(network: Network) -> (Self, Command<Self::Message>) {
        let stage = start::Start::new(network);
        (
            Self {
                state: State::Start(stage.0),
            },
            Command::batch(vec![
                font::load(ROBOTO_MONO_REGULAR_BYTES).map(Message::FontLoaded),
                font::load(ROBOTO_MONO_LIGHT_BYTES).map(Message::FontLoaded),
                font::load(ROBOTO_MONO_BOLD_BYTES).map(Message::FontLoaded),
                stage.1.map(|m| m.into()),
            ]),
        )
    }

    fn title(&self) -> String {
        let (title, network) = match &self.state {
            State::Start(auth) => (auth.title(), auth.ctx.network),
            State::App(app) => (app.title(), app.ctx.keechain.network()),
        };

        let mut title = if title.is_empty() {
            APP_NAME.to_string()
        } else {
            format!("{APP_NAME} - {title}")
        };

        if network != Network::Bitcoin {
            title.push_str(&format!(" [{network}]"));
        }

        title
    }

    fn theme(&self) -> Theme {
        match &self.state {
            State::Start(start) => start.theme().into(),
            State::App(app) => app.theme().into(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match (&mut self.state, message) {
            (State::Start(start), Message::Start(msg)) => {
                let (command, stage_to_move) = start.update(*msg);
                if let Some(stage) = stage_to_move {
                    *self = stage;
                    return Command::perform(async {}, |_| {
                        Message::App(Box::new(app::Message::Tick))
                    });
                }
                command.map(|m| m.into())
            }
            (State::App(app), Message::App(msg)) => match *msg {
                app::Message::Lock => {
                    let new = Self::new(app.ctx.keechain.network());
                    *self = new.0;
                    new.1
                }
                _ => app.update(*msg).map(|m| m.into()),
            },
            _ => Command::none(),
        }
    }

    fn view(&self) -> Element<Self::Message> {
        match &self.state {
            State::Start(start) => start.view().map(|m| m.into()),
            State::App(app) => app.view().map(|m| m.into()),
        }
    }
}

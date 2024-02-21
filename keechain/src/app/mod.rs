// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use iced::{Command, Element, Subscription};
use keechain_core::bitcoin::Network;
use keechain_core::KeeChain;

mod context;
mod message;
mod screen;

pub use self::context::{Context, Stage};
pub use self::message::Message;
use self::screen::*;
use crate::theme::Theme;

pub trait State {
    fn title(&self) -> String;

    fn update(&mut self, ctx: &mut Context, message: Message) -> Command<Message>;

    fn view(&self, ctx: &Context) -> Element<Message>;

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn load(&mut self, _ctx: &Context) -> Command<Message> {
        Command::none()
    }
}

pub fn new_state(ctx: &Context) -> Box<dyn State> {
    match &ctx.stage {
        Stage::Home => HomeState::new().into(),
        Stage::Sign => SignState::new().into(),
    }
}

pub struct App {
    state: Box<dyn State>,
    pub(crate) ctx: Context,
}

impl App {
    pub fn new(keechain: KeeChain) -> Self {
        let stage = Stage::default();
        let ctx = Context::new(stage, keechain);
        Self {
            state: new_state(&ctx),
            ctx,
        }
    }

    pub fn title(&self) -> String {
        match self.ctx.keechain.name() {
            Some(name) => format!("{} [{name}]", self.state.title()),
            None => self.state.title(),
        }
    }

    pub fn theme(&self) -> Theme {
        match self.ctx.keechain.network() {
            Network::Bitcoin => Theme::Mainnet,
            Network::Testnet => Theme::Testnet,
            Network::Signet => Theme::Signet,
            _ => Theme::Regtest,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::View(stage) => {
                self.ctx.set_stage(stage);
                self.state = new_state(&self.ctx);
                self.state.load(&self.ctx)
            }
            Message::Tick => self.state.update(&mut self.ctx, message),
            //Message::Clipboard(data) => clipboard::write(data),
            _ => self.state.update(&mut self.ctx, message),
        }
    }

    pub fn view(&self) -> Element<Message> {
        self.state.view(&self.ctx)
    }
}

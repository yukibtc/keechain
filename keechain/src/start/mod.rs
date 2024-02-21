// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use iced::{Command, Element, Subscription};
use keechain_core::bitcoin::Network;

mod context;
mod message;
pub mod screen;

pub use self::context::{Context, Stage};
pub use self::message::Message;
use self::screen::{GenerateState, OpenState, RestoreState};
use crate::app::App;
use crate::theme::Theme;
use crate::KeechainApp;

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

pub fn new_state(context: &Context) -> Box<dyn State> {
    match &context.stage {
        Stage::Open => OpenState::new().into(),
        Stage::New => GenerateState::new().into(),
        Stage::Restore => RestoreState::new().into(),
    }
}

pub struct Start {
    state: Box<dyn State>,
    pub(crate) ctx: Context,
}

impl Start {
    pub fn new(network: Network) -> (Self, Command<Message>) {
        let stage = Stage::default();
        let ctx = Context::new(stage, network);
        let app = Self {
            state: new_state(&ctx),
            ctx,
        };
        (app, Command::perform(async {}, move |_| Message::Load))
    }

    pub fn title(&self) -> String {
        self.state.title()
    }

    pub fn theme(&self) -> Theme {
        match self.ctx.network {
            Network::Bitcoin => Theme::Mainnet,
            Network::Testnet => Theme::Testnet,
            Network::Signet => Theme::Signet,
            _ => Theme::Regtest,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        self.state.subscription()
    }

    pub fn update(&mut self, message: Message) -> (Command<Message>, Option<KeechainApp>) {
        match message {
            Message::View(stage) => {
                self.ctx.set_stage(stage);
                self.state = new_state(&self.ctx);
                (self.state.load(&self.ctx), None)
            }
            Message::Load => (self.state.load(&self.ctx), None),
            Message::OpenResult(keechain) => {
                let app = App::new(*keechain);
                (
                    Command::none(),
                    Some(KeechainApp {
                        state: crate::State::App(Box::new(app)),
                    }),
                )
            }
            _ => (self.state.update(&mut self.ctx, message), None),
        }
    }

    pub fn view(&self) -> Element<Message> {
        self.state.view(&self.ctx)
    }
}

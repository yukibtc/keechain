// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use iced::widget::{Button, Column, Row, TextInput as NativeTextInput};

use super::Text;
use crate::constants::DEFAULT_FONT_SIZE;

pub struct TextInput<Message> {
    value: String,
    placeholder: String,
    label: Option<String>,
    password: bool,
    button: Option<Button<'static, Message>>,
    on_input: Option<Box<dyn Fn(String) -> Message>>,
    on_submit: Option<Message>,
}

impl<Message> TextInput<Message>
where
    Message: Clone + 'static,
{
    pub fn new<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            value: value.into(),
            placeholder: String::new(),
            label: None,
            password: false,
            button: None,
            on_input: None,
            on_submit: None,
        }
    }

    pub fn with_label<S>(label: S, value: S) -> Self
    where
        S: Into<String>,
    {
        Self::new(value).label(label)
    }

    pub fn placeholder<S>(self, placeholder: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            placeholder: placeholder.into(),
            ..self
        }
    }

    pub fn label<S>(mut self, label: S) -> Self
    where
        S: Into<String>,
    {
        self.label = Some(label.into());
        self
    }

    pub fn password(self) -> Self {
        Self {
            password: true,
            ..self
        }
    }

    pub fn button(self, btn: Button<'static, Message>) -> Self {
        Self {
            button: Some(btn),
            ..self
        }
    }

    pub fn on_input(self, on_input: impl Fn(String) -> Message + 'static) -> Self {
        Self {
            on_input: Some(Box::new(on_input)),
            ..self
        }
    }

    pub fn on_submit(self, message: Message) -> Self {
        Self {
            on_submit: Some(message),
            ..self
        }
    }

    pub fn view(self) -> Column<'static, Message> {
        let mut text_input = NativeTextInput::new(self.placeholder.as_str(), self.value.as_str())
            .padding(10)
            .size(DEFAULT_FONT_SIZE as f32);

        if self.password {
            text_input = text_input.password();
        }

        if let Some(on_input) = self.on_input {
            text_input = text_input.on_input(on_input);
        }

        if let Some(message) = self.on_submit {
            text_input = text_input.on_submit(message);
        }

        let mut input_row = Row::new().push(text_input);

        if let Some(btn) = self.button {
            input_row = input_row.push(btn).spacing(5);
        }

        let mut content = Column::new().spacing(5);

        if let Some(label) = self.label {
            content = content.push(Row::new().push(Text::new(label).view()));
        }

        content.push(input_row)
    }
}

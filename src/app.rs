use std::{result, str::FromStr};

use color_eyre::{
    eyre::{Context, Ok, Result},
    owo_colors::OwoColorize,
};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    prelude::*,
    style::{Color, Style},
    widgets::Block,
    Frame,
};
use tokio::sync::mpsc::Sender;

use crate::{
    component::Component,
    components::{body::Body, footer::Footer, header::Header, input_field::InputField},
    events::{key::Key, message::MessageResponse, Message, Transition},
};

// TODO: Merge mode and running to State { View, TextInput, Finishing ... }
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    View,
    TextInput,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Running {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Page {
    #[default]
    Containers,
}

#[derive(Debug)]
pub struct App {
    pub running: Running,
    pub page: Page,
    mode: Mode,
    header: Header,
    body: Body,
    footer: Footer,
    input_field: InputField,
}

impl App {
    pub async fn new(tx: Sender<Message<Key, Transition>>) -> Result<Self> {
        let page = Page::default();

        let body = Body::new(page.clone())
            .await
            .context("unable to create new body component")?;

        let app = Self {
            running: Running::default(),
            page,
            mode: Mode::default(),
            header: Header::default(),
            body,
            footer: Footer::default(),
            input_field: InputField::new(tx),
        };
        Ok(app)
    }

    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        match self.mode {
            Mode::View => self.update_view_mode(message).await,
            Mode::TextInput => self.update_text_mode(message).await,
        }
    }

    pub async fn transition(&mut self, transition: Transition) -> Result<MessageResponse> {
        let result = match transition {
            Transition::Quit => {
                self.running = Running::Done;
                MessageResponse::Consumed
            }
        };
        Ok(result)
    }

    async fn update_view_mode(&mut self, message: Key) -> Result<MessageResponse> {
        match self
            .body
            .update(message)
            .await
            .context("unable to update body")?
        {
            MessageResponse::Consumed => return Ok(MessageResponse::Consumed),
            _ => {}
        };

        match message {
            Key::Char('q') | Key::Char('Q') => {
                self.running = Running::Done;
                Ok(MessageResponse::Consumed)
            }
            Key::Char(':') => {
                self.set_mode(Mode::TextInput);
                Ok(MessageResponse::Consumed)
            }
            _ => Ok(MessageResponse::NotConsumed),
        }
    }

    async fn update_text_mode(&mut self, message: Key) -> Result<MessageResponse> {
        let result = match message {
            Key::Esc => {
                self.set_mode(Mode::View);
                MessageResponse::Consumed
            }
            _ => self.input_field.update(message).await.unwrap(),
        };
        Ok(result)
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode.clone();
        match mode {
            Mode::TextInput => self.input_field.initialise(),
            Mode::View => {}
        }
    }

    pub fn draw(&mut self, f: &mut Frame<'_>) {
        let layout: Layout;
        let header: Rect;

        let body: Rect;
        let footer: Rect;
        match self.mode {
            Mode::TextInput => {
                let text_input: Rect;
                layout = Layout::vertical([
                    Constraint::Length(5),
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ]);
                [header, text_input, body, footer] = layout.areas(f.size());
                self.input_field.draw(f, text_input);
            }
            _ => {
                layout = Layout::vertical([
                    Constraint::Length(5),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ]);
                [header, body, footer] = layout.areas(f.size());
            }
        }

        self.header.draw(f, header);
        self.body.draw(f, body);
        self.footer.draw(f, footer)
    }
}

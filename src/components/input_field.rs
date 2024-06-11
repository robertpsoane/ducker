use color_eyre::eyre::{Context, Result};
use ratatui::{
    layout::{Margin, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
};

use ratatui::prelude::*;
use tokio::sync::mpsc::Sender;

use crate::{
    component::Component,
    events::{message::MessageResponse, Key, Message, Transition},
    util::send_transition,
};

#[derive(Debug)]
pub struct InputField {
    input: String,
    prompt: char,
    tx: Sender<Message<Key, Transition>>,
}

impl InputField {
    pub fn new(tx: Sender<Message<Key, Transition>>) -> Self {
        Self {
            input: String::new(),
            prompt: '>',
            tx,
        }
    }

    pub fn initialise(&mut self) {
        self.input = String::new();
    }

    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        match message {
            Key::Char(c) => self.input.push(c),
            Key::Backspace => {
                self.input.pop();
            }
            Key::Enter => self
                .submit()
                .await
                .context("unable to submit user command")?,
            _ => return Ok(MessageResponse::NotConsumed),
        }

        Ok(MessageResponse::Consumed)
    }

    async fn submit(&mut self) -> Result<()> {
        let transition = match &*self.input {
            "quit" => Some(Transition::Quit),
            _ => None,
        };

        if let Some(t) = transition {
            send_transition(self.tx.clone(), t)
                .await
                .context("unable to send transition")?;
        }

        self.initialise();
        Ok(())
    }
}

impl Component for InputField {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::Plain)
            .padding(Padding::left(300));

        f.render_widget(block, area);

        let inner_body_margin = Margin::new(2, 1);
        let body_inner = area.inner(&inner_body_margin);

        let text = Line::from(vec![
            Span::styled::<String, Style>(self.prompt.into(), Style::new().green()),
            Span::raw::<String>(' '.into()),
            Span::raw(self.input.clone()),
        ]);
        let p = Paragraph::new(text);
        f.render_widget(p, body_inner)
    }
}

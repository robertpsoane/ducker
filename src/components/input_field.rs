use color_eyre::eyre::{Context, Result};
use ratatui::{
    layout::{Margin, Rect},
    prelude::*,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
};

use tokio::sync::mpsc::Sender;

use crate::{
    autocomplete::Autocomplete,
    events::{message::MessageResponse, Key, Message, Transition},
    traits::Component,
    util::send_transition,
};

const QUIT: &str = "quit";
const Q: &str = "q";
const IMAGE: &str = "image";
const IMAGES: &str = "images";
const CONTAINER: &str = "container";
const CONTAINERS: &str = "containers";

#[derive(Debug)]
pub struct InputField {
    input: String,
    prompt: char,
    tx: Sender<Message<Key, Transition>>,
    candidate: Option<String>,
    ac: Autocomplete<'static>,
}

impl InputField {
    pub fn new(tx: Sender<Message<Key, Transition>>) -> Self {
        Self {
            input: String::new(),
            prompt: '>',
            tx,
            candidate: None,
            ac: Autocomplete::from(vec![QUIT, Q, IMAGE, IMAGES, CONTAINER, CONTAINERS]),
        }
    }

    pub fn initialise(&mut self) {
        self.input = String::new();
        self.candidate = None;
    }

    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        match message {
            Key::Char(c) => {
                self.input.push(c);
                let input = &self.input;
                self.candidate = self.ac.get_completion(input);
            }
            Key::Tab => {
                if let Some(candidate) = &self.candidate {
                    self.input.clone_from(candidate)
                }
            }
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
            Q | QUIT => Some(Transition::Quit),
            IMAGE | IMAGES => Some(Transition::ToImagePage),
            CONTAINER | CONTAINERS => Some(Transition::ToContainerPage),
            _ => None,
        };

        if let Some(t) = transition {
            send_transition(self.tx.clone(), Transition::ToViewMode)
                .await
                .context("unable to send transition")?;
            send_transition(self.tx.clone(), t)
                .await
                .context("unable to send transition")?;
        }
        // At some point I want it to pop up a modal
        // else {
        //     panic!("")
        // }

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

        let mut input_text = vec![
            Span::styled::<String, Style>(format!("{} ", self.prompt), Style::new().green()),
            Span::raw(self.input.clone()),
        ];

        if let Some(candidate) = &self.candidate {
            if let Some(delta) = candidate.strip_prefix(&self.input as &str) {
                input_text.push(
                    Span::raw(delta).style(
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::DIM),
                    ),
                )
            }
        }

        let p = Paragraph::new(Line::from(input_text));
        f.render_widget(p, body_inner)
    }
}

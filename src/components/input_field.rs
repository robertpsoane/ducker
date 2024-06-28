use color_eyre::eyre::{Context, Result};
use itertools::min;
use ratatui::{
    layout::{Margin, Rect},
    prelude::*,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
};
use std::{collections::VecDeque, fmt::Debug};

use tokio::sync::mpsc::Sender;

use crate::{
    autocomplete::Autocomplete,
    context::AppContext,
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
    prompt: String,
    tx: Sender<Message<Key, Transition>>,
    candidate: Option<String>,
    ac: Autocomplete<'static>,
    history: History,
}

impl InputField {
    pub fn new(tx: Sender<Message<Key, Transition>>, prompt: String) -> Self {
        Self {
            input: String::new(),
            prompt,
            tx,
            candidate: None,
            ac: Autocomplete::from(vec![QUIT, Q, IMAGE, IMAGES, CONTAINER, CONTAINERS]),
            history: History::new(),
        }
    }

    pub fn initialise(&mut self) {
        self.input = String::new();
        self.candidate = None;
        self.history.reset_idx();
    }

    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        match message {
            Key::Char(c) => {
                self.history.reset_idx();
                self.input.push(c);
                let input = &self.input;
                self.candidate = self.ac.get_completion(input);
            }
            Key::Tab => {
                if let Some(candidate) = &self.candidate {
                    self.input.clone_from(candidate)
                }
            }
            Key::Up => {
                self.history.conditional_set_working_buffer(&self.input);
                if let Some(v) = &self.history.next() {
                    self.input.clone_from(v);
                }
            }
            Key::Down => {
                if let Some(v) = &self.history.previous() {
                    self.input.clone_from(v);
                }
            }
            Key::Backspace => {
                self.input.pop();
            }
            Key::Enter => {
                self.history.add_value(&self.input);
                self.history.reset_idx();
                self.submit()
                    .await
                    .context("unable to submit user command")?;
            }

            _ => return Ok(MessageResponse::NotConsumed),
        }

        Ok(MessageResponse::Consumed)
    }

    async fn submit(&mut self) -> Result<()> {
        let transition = match &*self.input {
            Q | QUIT => Some(Transition::Quit),
            IMAGE | IMAGES => Some(Transition::ToImagePage(AppContext::default())),
            CONTAINER | CONTAINERS => Some(Transition::ToContainerPage(AppContext::default())),
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
        let body_inner = area.inner(inner_body_margin);

        let mut input_text = vec![
            Span::styled::<String, Style>(format!("{} ", self.prompt), Style::new().green()),
            Span::raw(self.input.clone()),
        ];

        if let Some(candidate) = &self.candidate {
            if let Some(delta) = candidate.strip_prefix(&self.input as &str) {
                input_text
                    .push(Span::raw(delta).style(Style::default().add_modifier(Modifier::DIM)))
            }
        }

        let p = Paragraph::new(Line::from(input_text));
        f.render_widget(p, body_inner)
    }
}

const MAX_HISTORY_SIZE: usize = 100;

#[derive(Debug)]
struct History {
    values: VecDeque<String>,
    working_buffer: Option<String>,
    idx: Option<usize>,
}

impl Default for History {
    fn default() -> Self {
        Self {
            values: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            working_buffer: None,
            idx: None,
        }
    }
}

impl History {
    pub fn new() -> Self {
        let v = VecDeque::with_capacity(MAX_HISTORY_SIZE);
        Self {
            values: v,
            working_buffer: None,
            idx: None,
        }
    }

    pub fn add_value(&mut self, v: &str) {
        if v.trim().is_empty() {
            return;
        }
        if self.values.len() == MAX_HISTORY_SIZE {
            self.values.pop_back();
        }
        self.values.push_front(v.into());
    }

    pub fn reset_idx(&mut self) {
        self.idx = None;
        self.working_buffer = None;
    }

    pub fn next(&mut self) -> Option<String> {
        let mut next_idx = match self.idx {
            Some(idx) => idx + 1,
            None => 0,
        };

        let max_idx = self.values.len() - 1;

        if max_idx < next_idx {
            next_idx = max_idx
        };

        self.idx = Some(next_idx);
        self.values.get(next_idx).cloned()
    }

    pub fn previous(&mut self) -> Option<String> {
        let mut next_idx = match self.idx {
            None => return self.working_buffer.clone(),
            Some(idx) => {
                if idx == 0 {
                    self.idx = None;
                    return self.working_buffer.clone();
                } else {
                    idx - 1
                }
            }
        };

        next_idx = min([next_idx, self.values.len()]).unwrap();

        self.idx = Some(next_idx);
        self.values.get(next_idx).cloned()
    }

    /// Sets a working buffer for the history; in essence the buffer prior to
    /// querying the history.  Acts as a default when the index drops back "below zero"
    pub fn conditional_set_working_buffer(&mut self, working_buffer: &String) {
        // Only add to the working buffer if we're actually adding something
        if let Some(buf) = &self.working_buffer {
            if buf.trim().is_empty() {
                return;
            }
        }
        // Only add to the working buffer if we aren't reading history
        if self.idx.is_some() {
            return;
        }
        self.working_buffer = Some(working_buffer.to_owned())
    }
}

use color_eyre::eyre::{Context, Result};
use itertools::min;
use ratatui::{layout::Rect, Frame};
use std::{collections::VecDeque, fmt::Debug};

use tokio::sync::mpsc::Sender;

use crate::{
    context::AppContext,
    events::{message::MessageResponse, transition::send_transition, Key, Message, Transition},
    traits::Component,
    widgets::text_input::Autocomplete,
};

use super::text_input_wrapper::TextInputWrapper;

const QUIT: &str = "quit";
const Q: &str = "q";
const IMAGE: &str = "image";
const IMAGES: &str = "images";
const CONTAINER: &str = "container";
const CONTAINERS: &str = "containers";
const VOLUME: &str = "volume";
const VOLUMES: &str = "volumes";
const NETWORK: &str = "network";
const NETWORKS: &str = "networks";

#[derive(Debug)]
pub struct CommandInput {
    tx: Sender<Message<Key, Transition>>,
    history: History,
    text_input: TextInputWrapper,
}

impl CommandInput {
    pub fn new(
        tx: Sender<Message<Key, Transition>>,
        prompt: String,
        ac_minimum_length: usize,
    ) -> Self {
        let ac: Autocomplete = Autocomplete::new(
            vec![
                QUIT, Q, IMAGE, IMAGES, CONTAINER, CONTAINERS, VOLUME, VOLUMES, NETWORK, NETWORKS,
            ],
            ac_minimum_length,
        );
        Self {
            tx,
            history: History::new(),
            text_input: TextInputWrapper::new(prompt, Some(ac)),
        }
    }

    pub fn initialise(&mut self) {
        self.text_input.reset();
        self.history.reset_idx();
    }

    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        // NB - for now the CommandInput is over-riding bits of the TextInputWrapper
        // This should probably be fixed by a generic TextInput widget over a
        // trait which defines the interaction between the widget and its state struct
        // for now unlikely we'll need history for anything else, so autocomplete
        // as an optional first class citizen is fine & we'll see what happens
        //
        // Similarly, it could be that autocomplete varies, or we want different types
        // of autocomplete, which would trigger a refactor?
        match message {
            Key::Char(_) => {
                self.history.reset_idx();
                self.text_input.update(message)?;
            }
            Key::Up => {
                let input_value = self.text_input.get_value();
                self.history.conditional_set_working_buffer(&input_value);
                if let Some(v) = &self.history.next() {
                    self.text_input.set_input(v.clone());
                }
            }
            Key::Down => {
                if let Some(v) = &self.history.previous() {
                    self.text_input.set_input(v.clone());
                }
            }
            Key::Enter => {
                self.history.add_value(&self.text_input.get_value());
                self.history.reset_idx();
                self.submit()
                    .await
                    .context("unable to submit user command")?;
            }

            _ => return self.text_input.update(message),
        }

        Ok(MessageResponse::Consumed)
    }

    async fn submit(&mut self) -> Result<()> {
        let transition = match &*self.text_input.get_value() {
            Q | QUIT => Some(Transition::Quit),
            IMAGE | IMAGES => Some(Transition::ToImagePage(AppContext::default())),
            CONTAINER | CONTAINERS => Some(Transition::ToContainerPage(AppContext::default())),
            VOLUME | VOLUMES => Some(Transition::ToVolumePage(AppContext::default())),
            NETWORK | NETWORKS => Some(Transition::ToNetworkPage(AppContext::default())),
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

        self.initialise();
        Ok(())
    }
}

impl Component for CommandInput {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        self.text_input.draw(f, area);
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

        let n_values = self.values.len();

        if n_values == 0 {
            return None;
        }

        let max_idx = n_values - 1;

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

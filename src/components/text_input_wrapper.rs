use color_eyre::eyre::Result;

use crate::{
    events::{message::MessageResponse, Key},
    traits::Component,
    widgets::text_input::{Autocomplete, TextInput, TextInputState},
};

/// Component which TextInput widget providing baked in keyboard input handling
#[derive(Debug)]
pub struct TextInputWrapper {
    prompt: String,
    state: TextInputState,
}

impl TextInputWrapper {
    pub fn new(prompt: String, autocomplete: Option<Autocomplete<'static>>) -> Self {
        let state = TextInputState::new(autocomplete);
        Self { prompt, state }
    }

    pub fn reset(&mut self) {
        self.state.reset()
    }

    pub fn update(&mut self, message: Key) -> Result<MessageResponse> {
        match message {
            Key::Char(c) => self.state.push_character(c),
            Key::Tab => self.state.accept_autocomplete_candidate(),
            Key::Backspace => {
                self.state.pop_character();
            }
            _ => return Ok(MessageResponse::NotConsumed),
        }

        Ok(MessageResponse::Consumed)
    }

    pub fn get_value(&self) -> String {
        self.state.get_value().clone()
    }

    pub fn set_input(&mut self, value: String) {
        self.state.set_input(value);
    }
}

impl Component for TextInputWrapper {
    fn draw(&mut self, f: &mut ratatui::Frame<'_>, area: ratatui::prelude::Rect) {
        let text_input = TextInput::new(Some(self.prompt.clone()));
        f.render_stateful_widget(text_input, area, &mut self.state);
    }
}

use ratatui::{
    layout::Margin,
    prelude::*,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};
use std::fmt::Debug;

use ratatui::widgets::{StatefulWidget, Widget};

pub struct TextInput {
    prompt: Option<String>,
}

impl TextInput {
    pub fn new(prompt: Option<String>) -> Self {
        Self { prompt }
    }
}

impl StatefulWidget for TextInput {
    type State = TextInputState;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::Plain)
            .padding(Padding::left(300));

        block.render(area, buf);

        let inner_body_margin = Margin::new(2, 1);
        let body_inner = area.inner(inner_body_margin);

        let mut input_text = vec![];
        if let Some(prompt) = self.prompt.clone() {
            input_text.push(Span::styled::<String, Style>(
                format!("{} ", prompt),
                Style::new().green(),
            ));
        }
        input_text.push(Span::raw(&state.value));

        if let Some(candidate) = &state.candidate
            && let Some(delta) = candidate.strip_prefix(&state.value as &str)
        {
            input_text.push(Span::raw(delta).style(Style::default().add_modifier(Modifier::DIM)))
        }

        let p = Paragraph::new(Line::from(input_text));
        p.render(body_inner, buf)
    }
}

#[derive(Debug)]
pub struct TextInputState {
    value: String,
    candidate: Option<String>,
    autocomplete: Option<Autocomplete<'static>>,
}

impl TextInputState {
    pub fn new(autocomplete: Option<Autocomplete<'static>>) -> Self {
        Self {
            value: String::new(),
            candidate: None,
            autocomplete,
        }
    }

    pub fn reset(&mut self) {
        self.value = String::new();
        self.candidate = None;
    }

    pub fn set_input(&mut self, value: String) {
        self.value = value;
        self.update_autocomplete();
    }

    pub fn get_value(&self) -> String {
        self.value.clone()
    }

    pub fn push_character(&mut self, c: char) {
        self.value.push(c);
        self.update_autocomplete();
    }

    pub fn pop_character(&mut self) {
        self.value.pop();
        self.update_autocomplete();
    }

    pub fn accept_autocomplete_candidate(&mut self) {
        if let Some(c) = &self.candidate {
            self.value.clone_from(c)
        }
    }

    fn update_autocomplete(&mut self) {
        if let Some(ac) = &self.autocomplete {
            self.candidate = ac.get_completion(&self.value)
        }
    }
}

#[derive(Debug)]
pub struct Autocomplete<'a> {
    possibles: Vec<&'a str>,
    autocomplete_min_length: usize,
}

impl<'a> Autocomplete<'a> {
    pub fn new(mut possibles: Vec<&'a str>, autocomplete_min_length: usize) -> Self {
        possibles.sort();
        Self {
            possibles,
            autocomplete_min_length,
        }
    }

    pub fn get_completion(&self, current: &str) -> Option<String> {
        if current.len() < self.autocomplete_min_length {
            return None;
        }

        for possible in &self.possibles {
            if possible.starts_with(current) {
                let candidate = String::from(*possible);
                return Some(candidate);
            }
        }

        None
    }
}

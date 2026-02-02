use color_eyre::eyre::Result;
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Block, Borders},
};

use crate::{
    events::{Key, message::MessageResponse},
    traits::Component,
};

use super::text_input_wrapper::TextInputWrapper;

const SLASH_KEY: Key = Key::Char('/');
const ESC_KEY: Key = Key::Esc;
const ENTER_KEY: Key = Key::Enter;

#[derive(Debug)]
pub struct TableFilter {
    pub is_filtering: bool,
    pub input: TextInputWrapper,
}

impl Default for TableFilter {
    fn default() -> Self {
        Self {
            is_filtering: false,
            input: TextInputWrapper::new("Filter".to_string(), None),
        }
    }
}

impl TableFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_input(&mut self, message: Key) -> Result<Option<MessageResponse>> {
        if self.is_filtering {
            match message {
                ESC_KEY => {
                    self.is_filtering = false;
                    self.input.reset();
                    return Ok(Some(MessageResponse::Consumed));
                }
                ENTER_KEY => {
                    self.is_filtering = false;
                    return Ok(Some(MessageResponse::Consumed));
                }
                _ => {
                    self.input.update(message)?;
                    return Ok(Some(MessageResponse::Consumed));
                }
            }
        }

        if message == SLASH_KEY {
            self.is_filtering = true;
            return Ok(Some(MessageResponse::Consumed));
        }

        Ok(None)
    }

    pub fn text(&self) -> String {
        self.input.get_value().to_lowercase()
    }

    pub fn is_active(&self) -> bool {
        self.is_filtering || !self.input.get_value().is_empty()
    }
}

impl Component for TableFilter {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        if self.is_active() {
            // Check if we have enough height to draw a border
            if area.height > 2 {
                let block = Block::default().borders(Borders::ALL).title("Filter");
                let inner_area = block.inner(area);
                f.render_widget(block, area);
                self.input.draw(f, inner_area);
            } else {
                self.input.draw(f, area);
            }
        }
    }
}

use crate::config::Config;
use crate::{
    components::help::{PageHelp, PageHelpBuilder},
    context::AppContext,
    events::{message::MessageResponse, Key},
    traits::{Close, Component, Page},
};
use color_eyre::eyre::Result;
use ratatui::{
    layout::Rect,
    text::Text,
    widgets::{Block, Paragraph},
    Frame,
};
use std::sync::{Arc, Mutex};

const HELP_TEXT: &str = include_str!("../../README.md");

const UP_KEY: Key = Key::Up;
const DOWN_KEY: Key = Key::Down;
const J_KEY: Key = Key::Char('j');
const K_KEY: Key = Key::Char('k');
const G_KEY: Key = Key::Char('g');
const SHIFT_G_KEY: Key = Key::Char('G');

#[derive(Debug)]
pub struct HelpPage {
    scroll: u16,
    max_scroll: u16,
    help_text: &'static str,
}

#[async_trait::async_trait]
impl Page for HelpPage {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        match message {
            UP_KEY | K_KEY => {
                self.scroll_up(1);
                Ok(MessageResponse::Consumed)
            }
            DOWN_KEY | J_KEY => {
                self.scroll_down(1);
                Ok(MessageResponse::Consumed)
            }
            G_KEY => {
                self.scroll = 0;
                Ok(MessageResponse::Consumed)
            }
            SHIFT_G_KEY => {
                self.scroll = self.max_scroll;
                Ok(MessageResponse::Consumed)
            }
            _ => Ok(MessageResponse::NotConsumed),
        }
    }
    async fn initialise(&mut self, _cx: AppContext) -> Result<()> {
        Ok(())
    }
    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        let config = Arc::new(Config::default());
        let help = PageHelpBuilder::new("Help".to_string(), config)
            .add_input(G_KEY.to_string(), "top".to_string())
            .add_input(SHIFT_G_KEY.to_string(), "bottom".to_string())
            .build();
        Arc::new(Mutex::new(help))
    }
}

#[async_trait::async_trait]
impl Close for HelpPage {}

impl HelpPage {
    pub fn new() -> Self {
        let usage_start = HELP_TEXT.find("## Usage").unwrap_or(0);
        let config_start = HELP_TEXT.find("## Configuration").unwrap_or(0);
        let max_scroll: u16 = 100;
        Self {
            scroll: 0,
            help_text: &HELP_TEXT[usage_start..config_start],
            max_scroll,
        }
    }
    pub fn scroll_up(&mut self, amount: u16) {
        self.scroll = self.scroll.saturating_sub(amount);
    }
    pub fn scroll_down(&mut self, amount: u16) {
        self.scroll = (self.scroll.saturating_add(amount)).min(self.max_scroll);
    }
}

impl Default for HelpPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for HelpPage {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let block = Block::default();
        let text = Text::from(self.help_text);
        let paragraph = Paragraph::new(text)
            .block(block)
            .scroll((self.scroll, 0))
            .wrap(ratatui::widgets::Wrap { trim: true });
        // This is a slightly backwards hack to pass back the total number of lines
        // in the paragraph to the _next_ rendering tick
        let total_lines = paragraph.line_count(area.width) as u16;
        self.max_scroll = total_lines.saturating_sub(area.height);
        if self.scroll > self.max_scroll {
            self.scroll = self.max_scroll;
        }
        f.render_widget(paragraph, area);
    }
}

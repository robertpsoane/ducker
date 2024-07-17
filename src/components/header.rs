use std::sync::Arc;

use ratatui::{
    layout::{self, Margin, Rect},
    style::Style,
    Frame,
};
use tui_big_text::{BigText, PixelSize};

use crate::{config::Config, traits::Component};

#[derive(Debug)]
pub struct Header {
    config: Arc<Config>,
}

impl Header {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

impl Component for Header {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let big_text = match BigText::builder()
            .pixel_size(PixelSize::HalfHeight)
            .style(Style::default().fg(self.config.theme.title()))
            .lines(vec!["Ducker".into()])
            .alignment(layout::Alignment::Center)
            .build()
        {
            Ok(b) => b,
            _ => panic!("Ahhhh!"),
        };

        let area = area.inner(Margin {
            vertical: 0,
            horizontal: 2,
        });

        f.render_widget(big_text, area);
    }
}

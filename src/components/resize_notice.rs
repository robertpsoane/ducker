use std::sync::Arc;

use ratatui::{
    layout::{self},
    style::Style,
    text::{Line, Span, Text},
    widgets::{block::Title, Block},
};
use ratatui_macros::{horizontal, vertical};

use crate::{config::Config, traits::Component};

const MIN_ROWS: u16 = 20;
const MIN_COLS: u16 = 100;

#[derive(Debug)]
pub struct ResizeScreen {
    pub min_height: u16,
    pub min_width: u16,
    config: Arc<Config>,
}

impl ResizeScreen {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            min_width: MIN_COLS,
            min_height: MIN_ROWS,
            config,
        }
    }
}

impl Component for ResizeScreen {
    fn draw(&mut self, f: &mut ratatui::Frame<'_>, area: ratatui::prelude::Rect) {
        let [_, area, _] = horizontal![>=0, <=40, >=0].areas(area);

        let [_, area, _] = vertical![>=0, <=9, >=0].areas(area);

        let size = f.size();

        let height = size.height;
        let mut height_span = Span::from(format!("{}", size.height));

        let height_style = if height >= self.min_height {
            Style::default().fg(self.config.theme.success())
        } else {
            Style::default().fg(self.config.theme.error())
        };
        height_span = height_span.style(height_style);

        let width = size.width;
        let mut width_span = Span::from(format!("{}", size.width));

        let width_style = if width >= self.min_width {
            Style::default().fg(self.config.theme.success())
        } else {
            Style::default().fg(self.config.theme.error())
        };
        width_span = width_span.style(width_style);

        let messages = vec![
            Line::from("Terminal too small; current size:"),
            Line::from(vec![
                Span::from("Width = "),
                width_span,
                Span::from(", ".to_string()),
                Span::from("Height = "),
                height_span,
            ]),
            Line::from(""),
            Line::from("Required dimensions:"),
            Line::from(vec![
                Span::from(format!("Width = {}", self.min_width)),
                Span::from(", ".to_string()),
                Span::from(format!("Height = {}", self.min_height)),
            ]),
        ];

        let info = Text::from(messages).alignment(ratatui::layout::Alignment::Center);

        let block = Block::bordered()
            .title(Title::from("< Terminal Too Small >").alignment(layout::Alignment::Center))
            .border_style(Style::default().fg(self.config.theme.negative_highlight()));

        let [_, inner_area, _] = vertical![>=0, <=5, >=0].areas(area);

        f.render_widget(block, area);

        f.render_widget(info, inner_area)
    }
}

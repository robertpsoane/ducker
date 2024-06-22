use ratatui::{
    layout::{self, Constraint, Layout},
    style::Style,
    text::{Line, Span, Text},
    widgets::{block::Title, Block},
};

use crate::traits::Component;

const MIN_ROWS: u16 = 20;
const MIN_COLS: u16 = 80;

#[derive(Debug)]
pub struct ResizeScreen {
    pub min_height: u16,
    pub min_width: u16,
}

impl Default for ResizeScreen {
    fn default() -> Self {
        Self {
            min_width: MIN_COLS,
            min_height: MIN_ROWS,
        }
    }
}

impl Component for ResizeScreen {
    fn draw(&mut self, f: &mut ratatui::Frame<'_>, area: ratatui::prelude::Rect) {
        let [_, area, _] = Layout::horizontal(vec![
            Constraint::Min(0),
            Constraint::Max(40),
            Constraint::Min(0),
        ])
        .areas(area);

        let [_, area, _] = Layout::vertical(vec![
            Constraint::Min(0),
            Constraint::Max(9),
            Constraint::Min(0),
        ])
        .areas(area);

        let size = f.size();

        let height = size.height;
        let mut height_span = Span::from(format!("{}", size.height));

        let height_style = match height >= self.min_height {
            true => Style::default().fg(ratatui::style::Color::Green),
            false => Style::default().fg(ratatui::style::Color::Red),
        };
        height_span = height_span.style(height_style);

        let width = size.width;
        let mut width_span = Span::from(format!("{}", size.width));

        let width_style = match width >= self.min_width {
            true => Style::default().fg(ratatui::style::Color::Green),
            false => Style::default().fg(ratatui::style::Color::Red),
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
            .border_style(Style::default().fg(ratatui::style::Color::Magenta));

        let [_, inner_area, _] = Layout::vertical(vec![
            Constraint::Min(0),
            Constraint::Max(5),
            Constraint::Min(0),
        ])
        .areas(area);

        f.render_widget(block, area);

        f.render_widget(info, inner_area)
    }
}

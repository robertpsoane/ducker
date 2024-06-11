use ratatui::{
    layout::{self, Constraint, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{
        canvas::{Canvas, Circle, Line, Map, MapResolution, Rectangle},
        Block, Borders, Paragraph,
    },
    Frame,
};

use crate::component::Component;

#[derive(Default, Debug)]
pub struct Header {}

impl Component for Header {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let [left, right] =
            Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(area);

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());

        let title = Paragraph::new(Text::styled("Ducker", Style::default().fg(Color::Green)))
            .block(title_block);

        f.render_widget(title, left)
    }
}

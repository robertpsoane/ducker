use color_eyre::owo_colors::OwoColorize;
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
use tui_big_text::{BigText, PixelSize};

use crate::component::Component;

#[derive(Default, Debug)]
pub struct Header {}

impl Component for Header {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let big_text = match BigText::builder()
            .pixel_size(PixelSize::HalfHeight)
            .style(Style::default().fg(Color::Green))
            .lines(vec!["Ducker".into()])
            .alignment(layout::Alignment::Center)
            .build()
        {
            Ok(b) => b,
            _ => panic!("Ahhhh!"),
        };

        f.render_widget(big_text, area)
    }
}

use ratatui::{
    layout::{self, Margin, Rect},
    style::{Color, Style},
    Frame,
};
use tui_big_text::{BigText, PixelSize};

use crate::traits::Component;

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

        let area = area.inner(Margin {
            vertical: 0,
            horizontal: 2,
        });

        f.render_widget(big_text, area);
    }
}

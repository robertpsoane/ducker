use ratatui::{layout::Rect, Frame};

pub trait Component {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect);
}

use std::fmt::Debug;

use ratatui::{layout::Rect, Frame};

pub trait Component: Debug {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect);
}

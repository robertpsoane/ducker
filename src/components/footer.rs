use itertools::Itertools;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    Frame,
};

use crate::component::Component;

#[derive(Default, Debug)]
pub struct Footer {}

impl Footer {}

impl Component for Footer {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let keys = [
            ("K/↑", "Up"),
            ("J/↓", "Down"),
            ("Q/q", "Quit"),
            (":", "Command"),
        ];
        let spans = keys
            .iter()
            .flat_map(|(key, desc)| {
                let key = Span::styled(
                    format!(" <{key}> = "),
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::ITALIC),
                );
                let desc = Span::styled(
                    format!("{desc} "),
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::ITALIC),
                );
                [key, desc]
            })
            .collect_vec();

        let footer = Line::from(spans).centered().style(Style::new());

        f.render_widget(footer, area)
    }
}

use itertools::Itertools;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    Frame,
};

use crate::{config::Config, traits::Component};

#[derive(Debug)]
pub struct Footer {
    config: Box<Config>,
}

impl Footer {
    pub fn new(config: Box<Config>) -> Self {
        Self { config }
    }
}

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
                    Style::default()
                        .fg(self.config.theme.footer())
                        .add_modifier(Modifier::ITALIC),
                );
                let desc = Span::styled(
                    format!("{desc} "),
                    Style::default()
                        .fg(self.config.theme.footer())
                        .add_modifier(Modifier::ITALIC),
                );
                [key, desc]
            })
            .collect_vec();

        let footer = Line::from(spans).centered().style(Style::new());

        f.render_widget(footer, area)
    }
}

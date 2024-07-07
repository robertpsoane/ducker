use itertools::Itertools;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    Frame,
};

use crate::{config::Config, traits::Component};

use super::version::VersionComponent;

#[derive(Debug)]
pub struct Footer {
    config: Box<Config>,
    version: VersionComponent,
}

impl Footer {
    pub async fn new(config: Box<Config>) -> Self {
        Self {
            config: config.clone(),
            version: VersionComponent::new(config).await,
        }
    }
}

impl Component for Footer {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let layout = Layout::horizontal([
            Constraint::Length(20),
            Constraint::Min(0),
            Constraint::Length(20),
        ]);
        let [_left, mid, right] = layout.areas(area);

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

        f.render_widget(footer, mid);

        self.version.draw(f, right)
    }
}

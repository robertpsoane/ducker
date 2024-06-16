use itertools::Itertools;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::component::Component;

#[derive(Debug, Clone)]
pub struct PageHelp {
    name: String,
    inputs: Vec<(String, String)>,
}

impl PageHelp {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            inputs: vec![],
        }
    }

    pub fn add_input(mut self, trigger: String, description: String) -> Self {
        self.inputs.push((trigger, description));
        self
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl Component for PageHelp {
    fn draw(&mut self, f: &mut ratatui::Frame<'_>, area: ratatui::prelude::Rect) {
        let spans = self
            .inputs
            .iter()
            .flat_map(|(key, desc)| {
                let line = Line::from(vec![
                    Span::styled(
                        format!(" <{key}> = "),
                        Style::new().fg(Color::Red).add_modifier(Modifier::ITALIC),
                    ),
                    Span::styled(
                        format!("{desc} "),
                        Style::new().fg(Color::Red).add_modifier(Modifier::ITALIC),
                    ),
                ]);
                line
            })
            .collect_vec();

        let help_data = Paragraph::new(Line::from(spans).centered().style(Style::new()))
            .wrap(Wrap { trim: true });

        f.render_widget(help_data, area)
    }
}

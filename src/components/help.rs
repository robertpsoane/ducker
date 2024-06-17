use itertools::Itertools;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};

use crate::traits::Component;

#[derive(Debug, Clone)]
pub struct PageHelp {
    name: String,
    inputs: Vec<(String, String)>,
}

impl PageHelp {
    pub fn new(name: String) -> Self {
        Self {
            name,
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
            .map(|(key, desc)| {
                // let t = Text::from();

                Span::from(format!(" <{key}> = {desc} ")).style(
                    Style::default()
                        .add_modifier(Modifier::ITALIC)
                        .fg(Color::Red),
                )
            })
            .collect_vec();

        let help_data = Paragraph::new(Line::from(spans).centered().style(Style::new()))
            .wrap(Wrap { trim: true });

        f.render_widget(help_data, area)
    }
}

use itertools::Itertools;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::traits::Component;

#[derive(Debug, Clone)]
pub struct PageHelp {
    name: String,
    displays: Vec<String>,
    width: usize,
}

#[derive(Debug, Clone)]
pub struct PageHelpBuilder {
    name: String,
    inputs: Vec<(String, String)>,
}

impl PageHelpBuilder {
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

    pub fn build(mut self) -> PageHelp {
        self.inputs.sort_by_key(|(first, _)| first.to_owned());

        let mut width = 0;

        let displays = self
            .inputs
            .iter()
            .map(|(key, desc)| {
                let disp = format!(" <{key}> = {desc} ");
                if disp.len() > width {
                    width = disp.len();
                };

                disp
            })
            .collect_vec();

        PageHelp {
            name: self.name.clone(),
            displays,
            width,
        }
    }
}

impl PageHelp {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl Component for PageHelp {
    fn draw(&mut self, f: &mut ratatui::Frame<'_>, area: ratatui::prelude::Rect) {
        // This bit attempts to dynamically break the help bits of the display into a set of columns
        // These all get left-aligned on the far right
        let group_height = area.height - 1;

        // Integer division - round up
        let n_blocks =
            (self.displays.len() + (group_height as usize) - 1) / (group_height as usize);

        let displays = self.displays.clone();
        let width = self.width;

        let chunked_displays: Vec<&[String]> = displays.chunks(group_height as usize).collect();

        // Dynamically build horizontal of fixed width
        let mut constraints = vec![Constraint::Min(0)];
        for _ in 0..n_blocks {
            constraints.push(Constraint::Length(width as u16));
        }
        let columns = Layout::horizontal(constraints).split(f.size());

        // This slight monstrosity iterates over each chunk, builds the column then writes it to the
        // relevant buffer
        for (idx, display) in chunked_displays.iter().enumerate() {
            let column = Paragraph::new(
                display
                    .iter()
                    .map(|v| {
                        Line::from(
                            Span::from(format!("{v}\n")).style(
                                Style::default()
                                    .add_modifier(Modifier::ITALIC)
                                    .fg(Color::Red),
                            ),
                        )
                        .left_aligned()
                        .style(Style::new())
                    })
                    .collect_vec(),
            );
            f.render_widget(column, columns[idx + 1]);
        }
    }
}

use itertools::{max, Itertools};
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

use crate::traits::Component;

#[derive(Debug, Clone)]
pub struct PageHelp {
    name: String,
    inputs: Vec<(String, String)>,
    displays: Option<Vec<String>>,
    width: Option<usize>,
}

impl PageHelp {
    pub fn new(name: String) -> Self {
        Self {
            name,
            inputs: vec![],
            displays: None,
            width: None,
        }
    }

    pub fn add_input(mut self, trigger: String, description: String) -> Self {
        self.inputs.push((trigger, description));
        self
    }

    pub fn build(mut self) -> Self {
        self.inputs.sort_by_key(|(first, _)| first.to_owned());

        let mut max_width = 0;

        self.displays = Some(
            self.inputs
                .iter()
                .map(|(key, desc)| {
                    let disp = format!(" <{key}> = {desc} ");
                    if disp.len() > max_width {
                        max_width = disp.len();
                    };

                    disp
                })
                .collect_vec(),
        );
        self.width = Some(max_width);
        self
    }

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
        let n_blocks = (self.inputs.len() + (group_height as usize) - 1) / (group_height as usize);

        // If unset at run, want program to crash
        let displays = self.displays.clone().unwrap();
        let width = self.width.unwrap();

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

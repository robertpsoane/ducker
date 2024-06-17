use std::sync::Arc;

use futures::lock::Mutex;
use itertools::Itertools;

use color_eyre::eyre::Result;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{block::Title, Block, Clear, Paragraph, Wrap},
    Frame,
};

use crate::{
    events::{message::MessageResponse, Key},
    traits::{Callback, Component},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BooleanOptions {
    Yes,
    No,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum ModalState<T> {
    #[default]
    Invisible,
    Waiting(String),
    Complete(T),
}

#[derive(Default, Debug)]
pub struct ConfirmationModal<T> {
    pub state: ModalState<T>,
    title: String,
    callback: Option<Arc<Mutex<dyn Callback>>>,
}

impl<T> ConfirmationModal<T> {
    pub fn new(title: String) -> Self {
        Self {
            state: ModalState::default(),
            title,
            callback: None,
        }
    }

    pub fn initialise(&mut self, message: String, cb: Arc<Mutex<dyn Callback>>) {
        self.callback = Some(cb);
        self.state = ModalState::Waiting(message)
    }

    pub fn reset(&mut self) {
        self.callback = None;
        self.state = ModalState::Invisible
    }

    pub fn get_area(&self, area: Rect) -> Rect {
        let constraints = vec![
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ];

        let vertical_layout = Layout::vertical(constraints.clone());
        let horizontal_layout = Layout::horizontal(constraints.clone());

        let [_, mid, _] = vertical_layout.areas(area);
        let [_, area, _] = horizontal_layout.areas(mid);
        area
    }
}

impl ConfirmationModal<BooleanOptions> {
    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        match message {
            Key::Esc | Key::Char('n') | Key::Char('N') => {
                self.state = ModalState::Complete(BooleanOptions::No);
                Ok(MessageResponse::Consumed)
            }
            Key::Char('y') | Key::Char('Y') | Key::Enter => {
                self.state = ModalState::Complete(BooleanOptions::Yes);
                if let Some(cb) = self.callback.clone() {
                    cb.lock().await.call().await;
                }
                Ok(MessageResponse::Consumed)
            }
            // We don't want Q/q to be able to quit here
            Key::Char('q') | Key::Char('Q') => Ok(MessageResponse::Consumed),
            _ => Ok(MessageResponse::NotConsumed),
        }
    }
}

impl Component for ConfirmationModal<BooleanOptions> {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let message: String = match &self.state {
            ModalState::Waiting(v) => v.clone(),
            _ => return,
        };

        let area = self.get_area(area);

        let title = Title::from(format!("< {} >", self.title.clone())).alignment(Alignment::Center);

        let block = Block::bordered()
            .title(title)
            .border_style(Style::default())
            .style(Style::default());

        let inner_block = block.inner(area);

        f.render_widget(Clear, area);
        f.render_widget(block, area);

        let message = Paragraph::new(Text::from(message))
            .wrap(Wrap { trim: true })
            .centered();

        let vertical_layout = Layout::vertical(vec![
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Percentage(10),
            Constraint::Percentage(40),
        ]);

        let [_, top, _, bottom] = vertical_layout.areas(inner_block);

        f.render_widget(message, top);

        let keys = [
            // ("H/←", "Left"),
            // ("L/→", "Right"),
            ("Y/y/Enter", "Yes"),
            ("N/n", "No"),
        ];
        let spans = keys
            .iter()
            .flat_map(|(key, desc)| {
                let key = Span::styled(
                    format!(" <{key}> = "),
                    Style::new().add_modifier(Modifier::ITALIC),
                );
                let desc = Span::styled(
                    format!("{desc} "),
                    Style::new().add_modifier(Modifier::ITALIC),
                );
                [key, desc]
            })
            .collect_vec();

        let text = Line::from(spans).centered();

        f.render_widget(text, bottom);
    }
}

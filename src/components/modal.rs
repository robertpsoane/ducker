use std::{fmt::Debug, sync::Arc};

use futures::lock::Mutex;
use itertools::Itertools;

use color_eyre::eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Span, Text},
    widgets::{block::Title, Paragraph, Wrap},
    Frame,
};

use crate::{
    events::{message::MessageResponse, Key},
    traits::{Callback, Component, ModalComponent},
    widgets::modal::ModalWidget,
};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum ModalState {
    #[default]
    Closed,
    Open(String),
}

#[derive(Default, Debug)]
pub struct Modal<P> {
    pub discriminator: P,
    pub state: ModalState,
    title: String,
    callback: Option<Arc<Mutex<dyn Callback>>>,
}

impl<P> Modal<P> {
    pub fn new(title: String, discriminator: P) -> Self {
        Self {
            discriminator,
            state: ModalState::default(),
            title,
            callback: None,
        }
    }

    pub fn initialise(&mut self, message: String, cb: Option<Arc<Mutex<dyn Callback>>>) {
        self.callback = cb;
        self.state = ModalState::Open(message)
    }

    pub fn reset(&mut self) {
        self.callback = None;
        self.state = ModalState::Closed
    }
}

#[async_trait::async_trait]
impl<P> ModalComponent for Modal<P>
where
    P: Debug + Send,
{
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        match message {
            Key::Esc | Key::Char('n') | Key::Char('N') => {
                self.reset();
                Ok(MessageResponse::Consumed)
            }
            Key::Char('y') | Key::Char('Y') | Key::Enter => {
                if let Some(cb) = self.callback.clone() {
                    cb.lock().await.call().await?;
                }
                self.reset();
                Ok(MessageResponse::Consumed)
            }
            // We don't want Q to be able to quit here
            Key::Char('Q') | Key::Char('q') => Ok(MessageResponse::Consumed),
            _ => Ok(MessageResponse::NotConsumed),
        }
    }
}

impl<P> Component for Modal<P>
where
    P: std::fmt::Debug,
{
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let message: String = match &self.state {
            ModalState::Open(v) => v.clone(),
            _ => return,
        };

        let title = Title::from(format!("< {} >", self.title.clone())).alignment(Alignment::Center);

        let message = Paragraph::new(Text::from(message))
            .wrap(Wrap { trim: true })
            .centered();

        let spans = [("Y/y/Enter", "Yes"), ("N/n", "No")]
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

        let modal = ModalWidget::new(title, message, spans);

        f.render_widget(modal, area);
    }
}

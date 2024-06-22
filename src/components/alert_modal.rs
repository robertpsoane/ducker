use std::fmt::Debug;

use color_eyre::eyre::Result;
use ratatui::{
    layout::{Alignment, Rect},
    text::{Span, Text},
    widgets::{block::Title, Paragraph, Wrap},
    Frame,
};

use crate::{
    events::{message::MessageResponse, Key},
    traits::{Component, ModalComponent},
    widgets::modal::ModalWidget,
};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum ModalState {
    #[default]
    Closed,
    Open(String),
}

#[derive(Default, Debug)]
pub struct AlertModal<P> {
    pub discriminator: P,
    pub state: ModalState,
    title: String,
}

impl<P> AlertModal<P> {
    pub fn new(title: String, discriminator: P) -> Self {
        Self {
            discriminator,
            state: ModalState::default(),
            title,
        }
    }

    pub fn initialise(&mut self, message: String) {
        self.state = ModalState::Open(message)
    }

    pub fn reset(&mut self) {
        self.state = ModalState::Closed
    }
}

#[async_trait::async_trait]
impl<P> ModalComponent for AlertModal<P>
where
    P: Debug + Send,
{
    async fn update(&mut self, key: Key) -> Result<MessageResponse> {
        if let Key::Null = key {
            Ok(MessageResponse::Consumed)
        } else {
            // "Press any key to continue"
            self.reset();
            Ok(MessageResponse::Consumed)
        }
    }
}

impl<P> Component for AlertModal<P>
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

        let opt = Span::from("Press any key to continue...");

        let modal = ModalWidget::new(title, message, vec![opt]);

        f.render_widget(modal, area);
    }
}

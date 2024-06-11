use color_eyre::eyre::{Context, Result};
use ratatui::{
    layout::{Alignment, Margin, Rect},
    prelude::*,
    style::{Color, Style},
    text::Span,
    widgets::{block::Title, Block, Padding},
    Frame,
};

use crate::{
    app::Page,
    events::{message::MessageResponse, Key},
};

use crate::component::Component;

use super::containers::Containers;

#[derive(Debug)]
pub struct Body {
    page: Page,
    containers: Containers,
}

impl Body {
    pub async fn new(page: Page) -> Result<Self> {
        let docker = bollard::Docker::connect_with_socket_defaults()
            .context("unable to connect to local docker daemon")?;

        let containers = Containers::new(true, docker)
            .await
            .context("unable to create containers page")?;

        Ok(Body { page, containers })
    }

    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        match self.page {
            Page::Containers => self.containers.update(message).await,
        }
    }
}

impl Component for Body {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let title_message = match self.page {
            Page::Containers => self.containers.name.clone(),
        };

        let title = Title::from(format!("< {} >", title_message)).alignment(Alignment::Center);

        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::Plain)
            .title(title)
            .padding(Padding::left(300));

        f.render_widget(block, area);

        let inner_body_margin = Margin::new(2, 1);
        let body_inner = area.inner(&inner_body_margin);

        match self.page {
            Page::Containers => self.containers.draw(f, body_inner),
        }
    }
}

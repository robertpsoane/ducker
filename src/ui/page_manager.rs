use std::{
    any::Any,
    rc::Rc,
    sync::{Arc, Mutex},
};

use bollard::Docker;
use color_eyre::eyre::{Context, Result};
use futures::lock;
use ratatui::{
    layout::{Alignment, Margin, Rect},
    prelude::*,
    style::{Color, Style},
    text::Span,
    widgets::{block::Title, Block, Padding},
    Frame,
};

use crate::{
    component::Component,
    components::help::PageHelp,
    events::{message::MessageResponse, Key},
    page::Page,
    pages::containers::Containers,
    state,
};

#[derive(Debug)]
pub struct PageManager {
    current_page: state::CurrentPage,
    containers: Arc<Mutex<dyn Page>>,
}

impl PageManager {
    pub async fn new(page: state::CurrentPage) -> Result<Self> {
        let docker = bollard::Docker::connect_with_socket_defaults()
            .context("unable to connect to local docker daemon")?;

        let containers = Arc::new(Mutex::new(
            Containers::new(true, docker)
                .await
                .context("unable to create containers page")?,
        ));
        Ok(PageManager {
            current_page: page,
            containers,
        })
    }

    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        self.get_current_page()
            .lock()
            .unwrap()
            .update(message)
            .await
    }

    fn get_current_page(&self) -> Arc<Mutex<dyn Page>> {
        match self.current_page {
            state::CurrentPage::Containers => self.containers.clone(),
        }
    }

    pub fn draw_help(&mut self, f: &mut Frame<'_>, area: Rect) {
        self.get_current_page()
            .lock()
            .unwrap()
            .get_help()
            .lock()
            .unwrap()
            .draw(f, area);
    }
}

impl Component for PageManager {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let current_page = self.get_current_page();

        let title_message = current_page
            .lock()
            .unwrap()
            .get_help()
            .lock()
            .unwrap()
            .get_name();

        let title = Title::from(format!("< {} >", title_message)).alignment(Alignment::Center);

        let block = Block::bordered()
            .border_type(ratatui::widgets::BorderType::Plain)
            .title(title)
            .padding(Padding::left(300));

        f.render_widget(block, area);

        let inner_body_margin = Margin::new(2, 1);
        let body_inner = area.inner(&inner_body_margin);

        current_page.lock().unwrap().draw(f, body_inner);
    }
}

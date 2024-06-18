use std::sync::{Arc, Mutex};

use color_eyre::eyre::{Context, Result};
use ratatui::{
    layout::{Alignment, Margin, Rect},
    widgets::{block::Title, Block, Padding},
    Frame,
};
use tokio::sync::mpsc::Sender;

use crate::{
    events::{message::MessageResponse, Key, Message, Transition},
    pages::{
        containers::Containers,
        images::Images,
        logs::{self, Logs},
    },
    state,
    traits::{Component, Page},
};

#[derive(Debug)]
pub struct PageManager {
    current_page: state::CurrentPage,
    containers: Arc<Mutex<dyn Page>>,
    images: Arc<Mutex<dyn Page>>,
    logs: Arc<Mutex<dyn Page>>,
}

impl PageManager {
    pub async fn new(
        page: state::CurrentPage,
        tx: Sender<Message<Key, Transition>>,
    ) -> Result<Self> {
        let docker = bollard::Docker::connect_with_socket_defaults()
            .context("unable to connect to local docker daemon")?;

        let containers = Arc::new(Mutex::new(
            Containers::new(docker.clone(), tx.clone())
                .await
                .context("unable to create containers page")?,
        ));
        let images = Arc::new(Mutex::new(
            Images::new(docker.clone())
                .await
                .context("unable to create containers page")?,
        ));
        let logs = Arc::new(Mutex::new(
            Logs::new(docker, tx.clone())
                .await
                .context("unable to create containers page")?,
        ));

        let page_manager = Self {
            current_page: page,
            containers,
            images,
            logs,
        };

        page_manager
            .init()
            .await
            .context("failed to initialise page manager")?;

        Ok(page_manager)
    }

    pub async fn init(&self) -> Result<()> {
        self.get_current_page()
            .lock()
            .unwrap()
            .set_visible(self.current_page.clone())
            .await?;
        Ok(())
    }

    pub async fn transition(&mut self, transition: Transition) -> Result<MessageResponse> {
        let result = match transition {
            Transition::ToImagePage => {
                self.set_current_page(state::CurrentPage::Images).await?;
                MessageResponse::Consumed
            }
            Transition::ToContainerPage => {
                self.set_current_page(state::CurrentPage::Containers)
                    .await?;
                MessageResponse::Consumed
            }
            Transition::ToLogPage(container) => {
                self.set_current_page(state::CurrentPage::Logs(container))
                    .await?;
                MessageResponse::Consumed
            }
            _ => MessageResponse::NotConsumed,
        };
        Ok(result)
    }

    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        self.get_current_page()
            .lock()
            .unwrap()
            .update(message)
            .await
    }

    async fn set_current_page(&mut self, next_page: state::CurrentPage) -> Result<()> {
        if next_page == self.current_page {
            return Ok(());
        }
        self.get_current_page()
            .lock()
            .unwrap()
            .set_invisible()
            .await
            .context("unable to close old page")?;

        self.current_page = next_page.clone();

        self.get_current_page()
            .lock()
            .unwrap()
            .set_visible(next_page)
            .await
            .context("unable to open new page")?;

        Ok(())
    }

    fn get_current_page(&self) -> Arc<Mutex<dyn Page>> {
        match self.current_page {
            state::CurrentPage::Containers => self.containers.clone(),
            state::CurrentPage::Images => self.images.clone(),
            state::CurrentPage::Logs(_) => self.logs.clone(),
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

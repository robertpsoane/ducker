use std::sync::{Arc, Mutex};

use color_eyre::eyre::{bail, Ok, Result};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::Sender;

use crate::{
    components::help::PageHelp,
    docker::{container::DockerContainer, logs::DockerLogs},
    events::{
        message::{self, MessageResponse},
        Key, Message, Transition,
    },
    state::CurrentPage,
    traits::{Component, Page},
};

const NAME: &str = "Logs";

const ESC_KEY: Key = Key::Esc;

#[derive(Debug)]
pub struct Logs {
    docker: bollard::Docker,
    tx: Sender<Message<Key, Transition>>,
    logs: Option<DockerLogs>,
    page_help: Arc<Mutex<PageHelp>>,
}

impl Logs {
    pub async fn new(
        docker: bollard::Docker,
        tx: Sender<Message<Key, Transition>>,
    ) -> Result<Self> {
        let page_help = PageHelp::new(NAME.into()).add_input(format!("{ESC_KEY}"), "back".into());

        Ok(Self {
            docker,
            logs: None,
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
        })
    }
}

#[async_trait::async_trait]
impl Page for Logs {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        let res = match message {
            Key::Esc => {
                self.tx
                    .send(Message::Transition(Transition::ToContainerPage))
                    .await?;
                MessageResponse::Consumed
            }
            _ => MessageResponse::NotConsumed,
        };
        Ok(res)
    }
    async fn initialise(&mut self) -> Result<()> {
        Ok(())
    }
    async fn set_visible(&mut self, initial_state: CurrentPage) -> Result<()> {
        match initial_state {
            CurrentPage::Logs(container) => self.logs = Some(DockerLogs::from(container)),
            _ => bail!("Incorrect state passed to logs page"),
        }
        Ok(())
    }
    async fn set_invisible(&mut self) -> Result<()> {
        self.logs = None;
        Ok(())
    }
    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

impl Component for Logs {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {}
}

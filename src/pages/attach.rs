use std::sync::{Arc, Mutex};

use crossterm::terminal::disable_raw_mode;

use color_eyre::eyre::{bail, Ok, Result};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::Sender;

use crate::config::Config;
use crate::context::AppContext;
use crate::traits::Close;
use crate::{
    components::help::{PageHelp, PageHelpBuilder},
    docker::container::DockerContainer,
    events::{message::MessageResponse, Key, Message, Transition},
    traits::{Component, Page},
};

const NAME: &str = "Attach";

const ESC_KEY: Key = Key::Esc;

#[derive(Debug)]
pub struct Attach {
    config: Box<Config>,
    container: Option<DockerContainer>,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
}

impl Attach {
    pub fn new(tx: Sender<Message<Key, Transition>>, config: Box<Config>) -> Self {
        let page_help = PageHelpBuilder::new(NAME.into(), config.clone())
            .add_input(format!("{ESC_KEY}"), "back".into())
            .build();

        Self {
            config,
            container: None,
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
        }
    }
}

#[async_trait::async_trait]
impl Page for Attach {
    async fn update(&mut self, _message: Key) -> Result<MessageResponse> {
        let res = MessageResponse::Consumed;

        Ok(res)
    }

    async fn initialise(&mut self, cx: AppContext) -> Result<()> {
        if let Some(container) = cx.clone().docker_container {
            self.container = Some(container)
        } else {
            bail!("no docker container")
        }
        if let Some(container) = self.container.clone() {
            disable_raw_mode()?;
            container.attach(&self.config.default_exec).await?;

            let transition = if let Some(t) = cx.next() {
                t
            } else {
                Transition::ToContainerPage(AppContext {
                    docker_container: Some(container),
                    ..Default::default()
                })
            };

            self.tx.send(Message::Transition(transition)).await?;
            self.tx
                .send(Message::Transition(Transition::ToNewTerminal))
                .await?;
        }
        Ok(())
    }

    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

#[async_trait::async_trait]
impl Close for Attach {}

impl Component for Attach {
    fn draw(&mut self, _f: &mut Frame<'_>, _area: Rect) {}
}

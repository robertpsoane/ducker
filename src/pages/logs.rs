use futures::{future, stream, Stream, StreamExt};
use futures::{lock::Mutex as FutureMutex, FutureExt};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

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
    log_messages: Arc<Mutex<Vec<String>>>,
    log_streamer_handle: Option<JoinHandle<()>>,
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
            log_messages: Arc::new(Mutex::new(vec![])),
            log_streamer_handle: None,
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
        if let Some(logs) = &self.logs {
            let mut logs_stream = logs.get_log_stream(&self.docker, 10).await;
            let tx = self.tx.clone();
            let log_messages = self.log_messages.clone();
            self.log_streamer_handle = Some(tokio::spawn(async move {
                while let Some(v) = logs_stream.next().await {
                    {
                        log_messages.lock().unwrap().push(v);
                    }
                    let _ = tx.send(Message::Tick).await;
                }
            }));
        } else {
            bail!("unable to stream logs without logs to stream");
        }
        Ok(())
    }
    async fn set_visible(&mut self, initial_state: CurrentPage) -> Result<()> {
        match initial_state {
            CurrentPage::Logs(container) => self.logs = Some(DockerLogs::from(container)),
            _ => bail!("Incorrect state passed to logs page"),
        }
        self.initialise().await?;
        Ok(())
    }
    async fn set_invisible(&mut self) -> Result<()> {
        if let Some(handle) = &self.log_streamer_handle {
            handle.abort()
        }
        self.log_streamer_handle = None;
        self.logs = None;
        Ok(())
    }
    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

impl Component for Logs {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let v: String;
        {
            let lock = self.log_messages.lock().unwrap();
            if lock.len() > 1 {
                v = lock[lock.len() - 1].clone()
            } else {
                return;
            }
        }
        println!("{v}");
    }
}

use ansi_to_tui::IntoText;
use futures::StreamExt;
use ratatui::text::Text;
use ratatui::widgets::{List, ListState};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

use color_eyre::eyre::{bail, Ok, Result};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::Sender;

use crate::config::Config;
use crate::context::AppContext;
use crate::{
    components::help::{PageHelp, PageHelpBuilder},
    docker::{container::DockerContainer, logs::DockerLogs},
    events::{message::MessageResponse, Key, Message, Transition},
    traits::{Component, Page},
};

const NAME: &str = "Logs";

const ESC_KEY: Key = Key::Esc;
const J_KEY: Key = Key::Char('j');
const UP_KEY: Key = Key::Up;
const K_KEY: Key = Key::Char('k');
const DOWN_KEY: Key = Key::Down;
const SPACE_BAR: Key = Key::Char(' ');

#[derive(Debug)]
pub struct Logs {
    config: Box<Config>,
    docker: bollard::Docker,
    tx: Sender<Message<Key, Transition>>,
    container: Option<DockerContainer>,
    logs: Option<DockerLogs>,
    page_help: Arc<Mutex<PageHelp>>,
    log_messages: Arc<Mutex<Vec<String>>>,
    log_streamer_handle: Option<JoinHandle<()>>,
    list_state: ListState,
    auto_scroll: bool,
}

impl Logs {
    pub fn new(
        docker: bollard::Docker,
        tx: Sender<Message<Key, Transition>>,
        config: Box<Config>,
    ) -> Self {
        let page_help = Self::build_page_help(config.clone()).build();

        Self {
            config,
            docker,
            container: None,
            logs: None,
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
            log_messages: Arc::new(Mutex::new(vec![])),
            log_streamer_handle: None,
            list_state: ListState::default(),
            auto_scroll: true,
        }
    }

    fn build_page_help(config: Box<Config>) -> PageHelpBuilder {
        PageHelpBuilder::new(NAME.into(), config).add_input(format!("{ESC_KEY}"), "back".into())
    }

    fn increment_list(&mut self) {
        let current_idx = self.list_state.selected();
        match current_idx {
            None => self.list_state.select(Some(0)),
            Some(current_idx) => {
                let lock = self.log_messages.lock().unwrap();
                if !lock.is_empty() && current_idx < lock.len() - 1 {
                    self.list_state.select(Some(current_idx + 1))
                }
            }
        }
    }

    fn decrement_list(&mut self) {
        let current_idx = self.list_state.selected();
        match current_idx {
            None => self.list_state.select(Some(0)),
            Some(current_idx) => {
                if current_idx > 0 {
                    self.list_state.select(Some(current_idx - 1))
                }
            }
        }
    }

    fn activate_auto_scroll(&mut self) {
        if self.auto_scroll {
            return;
        }
        self.auto_scroll = true;
        self.page_help = Arc::new(Mutex::new(
            Self::build_page_help(self.config.clone()).build(),
        ));
    }

    fn deactivate_auto_scroll(&mut self) {
        if !self.auto_scroll {
            return;
        }
        self.auto_scroll = false;
        self.page_help = Arc::new(Mutex::new(
            Self::build_page_help(self.config.clone())
                .add_input(format!("{SPACE_BAR}"), "auto-scroll".into())
                .build(),
        ));
    }
}

#[async_trait::async_trait]
impl Page for Logs {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        let res = match message {
            Key::Esc => {
                self.tx
                    .send(Message::Transition(Transition::ToContainerPage(
                        AppContext {
                            docker_container: self.container.clone(),
                            ..Default::default()
                        },
                    )))
                    .await?;
                MessageResponse::Consumed
            }
            J_KEY | DOWN_KEY => {
                self.increment_list();
                self.deactivate_auto_scroll();
                MessageResponse::Consumed
            }
            K_KEY | UP_KEY => {
                self.decrement_list();
                self.deactivate_auto_scroll();
                MessageResponse::Consumed
            }
            SPACE_BAR => {
                self.activate_auto_scroll();
                MessageResponse::Consumed
            }
            _ => MessageResponse::NotConsumed,
        };

        let n_messages = self.log_messages.lock().unwrap().len();
        if self.auto_scroll && n_messages > 0 {
            self.list_state.select(Some(n_messages - 1));
        }
        Ok(res)
    }

    async fn initialise(&mut self) -> Result<()> {
        self.auto_scroll = true;
        if let Some(logs) = &self.logs {
            let mut logs_stream = logs.get_log_stream(&self.docker, 50);
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

    async fn set_visible(&mut self, cx: AppContext) -> Result<()> {
        if let Some(container) = cx.docker_container {
            self.logs = Some(DockerLogs::from(container.clone()));
            self.container = Some(container);
        } else {
            bail!("no docker container")
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
        self.log_messages = Arc::new(Mutex::new(vec![]));
        Ok(())
    }

    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

impl Component for Logs {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let logs: Vec<Text> = self
            .log_messages
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .map(|s| s.into_text().unwrap())
            .collect();
        let mut list = List::new(logs);

        if !self.auto_scroll {
            list = list.highlight_symbol("> ");
        }

        f.render_stateful_widget(list, area, &mut self.list_state)
    }
}

use ansi_to_tui::IntoText;
use futures::StreamExt;
use ratatui::text::Text;
use ratatui::widgets::{List, ListState};
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

use color_eyre::eyre::{Ok, Result, bail};
use ratatui::{Frame, layout::Rect};
use tokio::sync::mpsc::Sender;

use crate::config::Config;
use crate::context::AppContext;
use crate::docker::logs::StreamOptions;
use crate::{
    components::help::{PageHelp, PageHelpBuilder},
    docker::logs::DockerLogs,
    events::{Key, Message, Transition, message::MessageResponse},
    traits::{Close, Component, Page},
};

const NAME: &str = "Logs";

const ESC_KEY: Key = Key::Esc;
const J_KEY: Key = Key::Char('j');
const A_KEY: Key = Key::Char('a');
const UP_KEY: Key = Key::Up;
const K_KEY: Key = Key::Char('k');
const DOWN_KEY: Key = Key::Down;
const PAGE_UP_KEY: Key = Key::PageUp;
const PAGE_DOWN_KEY: Key = Key::PageDown;
const SPACE_BAR: Key = Key::Char(' ');
const G_KEY: Key = Key::Char('g');
const SHIFT_G_KEY: Key = Key::Char('G');

#[derive(Debug)]
pub struct Logs {
    config: Arc<Config>,
    docker: bollard::Docker,
    tx: Sender<Message<Key, Transition>>,
    logs: Option<DockerLogs>,
    page_help: Arc<Mutex<PageHelp>>,
    log_messages: Arc<Mutex<Vec<String>>>,
    log_streamer_handle: Option<JoinHandle<()>>,
    list_state: ListState,
    auto_scroll: bool,
    next: Option<Transition>,
    stream_options: StreamOptions,
    list_height: u16,
}

impl Logs {
    pub fn new(
        docker: bollard::Docker,
        tx: Sender<Message<Key, Transition>>,
        config: Arc<Config>,
    ) -> Self {
        let page_help = Self::build_page_help(NAME, config.clone()).build();

        Self {
            config,
            docker,
            logs: None,
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
            log_messages: Arc::new(Mutex::new(vec![])),
            log_streamer_handle: None,
            list_state: ListState::default(),
            auto_scroll: true,
            next: None,
            stream_options: StreamOptions::default(),
            list_height: 0,
        }
    }

    fn build_page_help(name: &str, config: Arc<Config>) -> PageHelpBuilder {
        PageHelpBuilder::new(format!("{} ({})", NAME, name), config)
            .add_input(format!("{ESC_KEY}"), "back".into())
            .add_input(format!("{G_KEY}"), "top".into())
            .add_input(format!("{SHIFT_G_KEY}"), "bottom".into())
            .add_input(format!("{A_KEY}"), "<all>".into())
    }

    fn activate_auto_scroll(&mut self) {
        if self.auto_scroll {
            return;
        }
        self.auto_scroll = true;

        self.page_help = Arc::new(Mutex::new(
            Self::build_page_help(
                if let Some(l) = &self.logs {
                    &l.container.names
                } else {
                    ""
                },
                self.config.clone(),
            )
            .build(),
        ));
    }

    fn deactivate_auto_scroll(&mut self) {
        if !self.auto_scroll {
            return;
        }
        self.auto_scroll = false;
        self.page_help = Arc::new(Mutex::new(
            Self::build_page_help(
                if let Some(l) = &self.logs {
                    &l.container.names
                } else {
                    ""
                },
                self.config.clone(),
            )
            .add_input(format!("{SPACE_BAR}"), "auto-scroll".into())
            .build(),
        ));
    }

    fn abort(&mut self) {
        if let Some(handle) = &self.log_streamer_handle {
            handle.abort()
        }
        self.log_streamer_handle = None;
        self.log_messages = Arc::new(Mutex::new(vec![String::new()]));
        self.logs = None;
    }

    fn scroll_down(&mut self, amount: usize) {
        let len = self.log_messages.lock().unwrap().len();
        if len == 0 {
            self.list_state.select(Some(0));
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = (current + amount).min(len - 1);
        self.list_state.select(Some(next));
        self.deactivate_auto_scroll();
    }

    fn scroll_up(&mut self, amount: usize) {
        let current = self.list_state.selected().unwrap_or(0);
        let next = current.saturating_sub(amount);
        self.list_state.select(Some(next));
        self.deactivate_auto_scroll();
    }

    async fn start_log_stream(&mut self) -> Result<()> {
        self.auto_scroll = true;
        if let Some(logs) = self.logs.clone() {
            let mut logs_stream = logs.get_log_stream(&self.docker, self.stream_options.clone());
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
}

#[async_trait::async_trait]
impl Page for Logs {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        let res = match message {
            Key::Esc => {
                let transition = if let Some(t) = self.next.clone() {
                    t
                } else if let Some(logs) = &self.logs {
                    Transition::ToContainerPage(AppContext {
                        docker_container: Some(logs.container.clone()),
                        ..Default::default()
                    })
                } else {
                    Transition::ToContainerPage(AppContext::default())
                };

                self.tx.send(Message::Transition(transition)).await?;
                MessageResponse::Consumed
            }
            G_KEY => {
                self.list_state.select_first();
                MessageResponse::Consumed
            }
            SHIFT_G_KEY => {
                self.list_state.select_last();
                MessageResponse::Consumed
            }
            J_KEY | DOWN_KEY => {
                self.scroll_down(1);
                MessageResponse::Consumed
            }
            PAGE_DOWN_KEY => {
                self.scroll_down(self.list_height.into());
                MessageResponse::Consumed
            }
            K_KEY | UP_KEY => {
                self.scroll_up(1);
                MessageResponse::Consumed
            }
            PAGE_UP_KEY => {
                self.scroll_up(self.list_height.into());
                MessageResponse::Consumed
            }
            SPACE_BAR => {
                self.activate_auto_scroll();
                MessageResponse::Consumed
            }
            A_KEY => {
                self.stream_options.all = true;
                let logs = self.logs.clone();
                self.abort();
                if let Some(l) = logs {
                    self.logs = Some(DockerLogs::from(l.container.clone()));
                }
                self.start_log_stream().await?;
                MessageResponse::Consumed
            }
            _ => MessageResponse::NotConsumed,
        };

        if self.auto_scroll {
            self.list_state.select_last();
        }
        Ok(res)
    }

    async fn initialise(&mut self, cx: AppContext) -> Result<()> {
        if let Some(container) = cx.clone().docker_container {
            self.logs = Some(DockerLogs::from(container.clone()));
        } else {
            bail!("no docker container")
        }

        if let Some(t) = cx.next() {
            self.next = Some(t)
        }

        self.start_log_stream().await?;

        Ok(())
    }

    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

#[async_trait::async_trait]
impl Close for Logs {
    async fn close(&mut self) -> Result<()> {
        self.abort();
        self.logs = None;
        self.log_messages = Arc::new(Mutex::new(vec![]));
        Ok(())
    }
}

impl Component for Logs {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        self.list_height = area.height.saturating_sub(1);
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

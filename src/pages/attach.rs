use ansi_to_tui::IntoText;
use futures::{future, stream, Stream, StreamExt};
use futures::{lock::Mutex as FutureMutex, FutureExt};
use ratatui::text::Text;
use ratatui::widgets::{List, ListState};
use std::sync::{Arc, Mutex};

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::{
    io::stdout,
    panic::{set_hook, take_hook},
};

use tokio::task::JoinHandle;

use color_eyre::eyre::{bail, Ok, Result};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::Sender;

use crate::terminal;
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

const NAME: &str = "Attach";

const ESC_KEY: Key = Key::Esc;
const J_KEY: Key = Key::Char('j');
const UP_KEY: Key = Key::Up;
const K_KEY: Key = Key::Char('k');
const DOWN_KEY: Key = Key::Down;
const SPACE_BAR: Key = Key::Char(' ');

#[derive(Debug)]
pub struct Attach {
    container: Option<DockerContainer>,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
}

impl Attach {
    pub async fn new(tx: Sender<Message<Key, Transition>>) -> Result<Self> {
        let page_help = PageHelp::new(NAME.into()).add_input(format!("{ESC_KEY}"), "back".into());

        Ok(Self {
            container: None,
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
        })
    }
}

#[async_trait::async_trait]
impl Page for Attach {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        let res = match message {
            _ => MessageResponse::NotConsumed,
        };

        Ok(res)
    }

    async fn initialise(&mut self) -> Result<()> {
        if let Some(container) = self.container.clone() {
            disable_raw_mode()?;
            container.attach("/bin/bash").await?;
            self.tx
                .send(Message::Transition(Transition::ToContainerPage))
                .await?;
            self.tx
                .send(Message::Transition(Transition::ToNewTerminal))
                .await?;
        }
        Ok(())
    }

    async fn set_visible(&mut self, initial_state: CurrentPage) -> Result<()> {
        match initial_state {
            CurrentPage::Attach(container) => self.container = Some(container),
            _ => bail!("Incorrect state passed to logs page"),
        }
        self.initialise().await?;
        Ok(())
    }

    async fn set_invisible(&mut self) -> Result<()> {
        Ok(())
    }

    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

impl Component for Attach {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {}
}

use std::sync::{Arc, Mutex};

use crossterm::terminal::disable_raw_mode;

use color_eyre::eyre::{bail, Result};
use ratatui::layout::{Constraint, Layout};
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::Sender;

use crate::components::alert_modal::{AlertModal, ModalState};
use crate::components::text_input_wrapper::TextInputWrapper;
use crate::config::Config;
use crate::context::AppContext;
use crate::docker::traits::Describe;
use crate::traits::{Close, ModalComponent};
use crate::{
    components::help::{PageHelp, PageHelpBuilder},
    docker::container::DockerContainer,
    events::{message::MessageResponse, Key, Message, Transition},
    traits::{Component, Page},
};

const NAME: &str = "Attach";
const PROMPT: &str = "exec: ";

const ESC_KEY: Key = Key::Esc;
const ENTER: Key = Key::Enter;

#[derive(Debug)]
enum ModalType {
    AlertModal,
}

#[derive(Debug)]
pub struct Attach {
    config: Box<Config>,
    container: Option<DockerContainer>,
    next: Option<Transition>,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
    attach_input: TextInputWrapper,
    alert_modal: AlertModal<ModalType>,
}

impl Attach {
    pub fn new(tx: Sender<Message<Key, Transition>>, config: Box<Config>) -> Self {
        let page_help = Self::build_page_help(config.clone(), None);
        Self {
            config,
            container: None,
            next: None,
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
            attach_input: TextInputWrapper::new(PROMPT.into(), None),
            alert_modal: AlertModal::new("Error".into(), ModalType::AlertModal),
        }
    }

    pub fn build_page_help(config: Box<Config>, name: Option<String>) -> PageHelp {
        PageHelpBuilder::new(
            match name {
                Some(n) => n,
                None => NAME.into(),
            },
            config.clone(),
        )
        .add_input(format!("{ESC_KEY}"), "back".into())
        .add_input(format!("{ENTER}"), "exec".into())
        .build()
    }

    async fn to_containers(&self) -> Result<()> {
        let transition = if let Some(t) = &self.next {
            t.clone()
        } else {
            Transition::ToContainerPage(AppContext::default())
        };

        self.tx.send(Message::Transition(transition)).await?;
        self.tx
            .send(Message::Transition(Transition::ToNewTerminal))
            .await?;

        Ok(())
    }

    async fn attach(&self, exec: &str) -> Result<()> {
        if self.container.is_none() {
            bail!("could not attach to a container when no container is set");
        };

        let container = self.container.clone().unwrap();

        disable_raw_mode()?;
        let res = container.attach(exec).await;
        self.tx
            .send(Message::Transition(Transition::ToNewTerminal))
            .await?;

        if res.is_err() {
            bail!("failed to attach to container");
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Page for Attach {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        if let ModalState::Open(_) = self.alert_modal.state.clone() {
            return self.alert_modal.update(message).await;
        }

        let res = match message {
            Key::Enter => {
                let exec = self.attach_input.get_value();
                match self.attach(&exec).await {
                    Ok(()) => self.to_containers().await?,
                    Err(_) => {
                        let msg = format!("Error in exec with command\n`{exec}`");
                        self.alert_modal.initialise(msg)
                    }
                }

                MessageResponse::Consumed
            }
            Key::Esc => {
                self.to_containers().await?;
                MessageResponse::Consumed
            }
            _ => self.attach_input.update(message)?,
        };
        Ok(res)
    }

    async fn initialise(&mut self, cx: AppContext) -> Result<()> {
        if let Some(container) = cx.clone().docker_container {
            let page_name = format!("{NAME} ({})", container.get_name());
            self.page_help = Arc::new(Mutex::new(Self::build_page_help(
                self.config.clone(),
                Some(page_name),
            )));
            self.container = Some(container)
        } else {
            bail!("no docker container")
        }
        self.attach_input
            .set_input(self.config.default_exec.clone());
        self.next = cx.next();
        Ok(())
    }

    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

#[async_trait::async_trait]
impl Close for Attach {}

impl Component for Attach {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let height = area.height;

        let [_, text_area, _] = Layout::vertical([
            Constraint::Length((height - 3) / 2),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .areas(area);

        let [_, text_area, _] = Layout::horizontal([
            Constraint::Percentage(5),
            Constraint::Percentage(90),
            Constraint::Percentage(5),
        ])
        .areas(text_area);

        self.attach_input.draw(f, text_area);

        if let ModalState::Open(_) = self.alert_modal.state.clone() {
            self.alert_modal.draw(f, area)
        }
    }
}

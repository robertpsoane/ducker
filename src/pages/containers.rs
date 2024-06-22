use bollard::Docker;
use color_eyre::eyre::{bail, Context, Result};
use futures::lock::Mutex as FutureMutex;
use ratatui::{
    layout::Rect,
    prelude::*,
    style::Style,
    widgets::{Row, Table, TableState},
    Frame,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;

use crate::{
    callbacks::DeleteContainer,
    components::{
        boolean_modal::{BooleanModal, ModalState},
        help::PageHelp,
    },
    context::AppContext,
    docker::container::DockerContainer,
    events::{message::MessageResponse, Key, Message, Transition},
    traits::{Component, ModalComponent, Page},
};

const NAME: &str = "Containers";

const UP_KEY: Key = Key::Up;
const DOWN_KEY: Key = Key::Down;

const A_KEY: Key = Key::Char('a');
const J_KEY: Key = Key::Char('j');
const K_KEY: Key = Key::Char('k');
const CTRL_D_KEY: Key = Key::Ctrl('d');
const R_KEY: Key = Key::Char('r');
const S_KEY: Key = Key::Char('s');
const G_KEY: Key = Key::Char('g');
const L_KEY: Key = Key::Char('l');
const SHIFT_G_KEY: Key = Key::Char('G');

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ModalTypes {
    DeleteContainer,
}

#[derive(Debug)]
pub struct Containers {
    pub name: String,
    pub visible: bool,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
    docker: Docker,
    containers: Vec<DockerContainer>,
    list_state: TableState,
    modal: Option<BooleanModal<ModalTypes>>,
}

#[async_trait::async_trait]
impl Page for Containers {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        if !self.visible {
            return Ok(MessageResponse::NotConsumed);
        }

        // If a modal is open, we process it; if it is open or complete, and the
        // result is Consumed, we exit early with the Consumed result
        if let Some(m) = self.modal.as_mut() {
            if let ModalState::Open(_) = m.state {
                let res = m.update(message).await;
                if let ModalState::Closed = m.state {
                    self.modal = None;
                }
                return res;
            }
        }

        let result = match message {
            UP_KEY | K_KEY => {
                self.decrement_list();
                MessageResponse::Consumed
            }
            DOWN_KEY | J_KEY => {
                self.increment_list();
                MessageResponse::Consumed
            }
            CTRL_D_KEY => match self.delete_container() {
                Ok(_) => MessageResponse::Consumed,
                Err(_) => MessageResponse::NotConsumed,
            },
            R_KEY => {
                self.start_container()
                    .await
                    .context("could not start container")?;
                MessageResponse::Consumed
            }
            S_KEY => {
                self.stop_container()
                    .await
                    .context("could not stop container")?;
                MessageResponse::Consumed
            }
            G_KEY => {
                self.list_state.select(Some(0));
                MessageResponse::Consumed
            }
            SHIFT_G_KEY => {
                self.list_state.select(Some(self.containers.len() - 1));
                MessageResponse::Consumed
            }
            A_KEY => {
                let container = self.get_container()?;
                self.tx
                    .send(Message::Transition(Transition::ToAttach(AppContext {
                        docker_container: Some(container.clone()),
                        ..Default::default()
                    })))
                    .await?;
                MessageResponse::Consumed
            }
            L_KEY => {
                let container = self.get_container()?;
                self.tx
                    .send(Message::Transition(Transition::ToLogPage(AppContext {
                        docker_container: Some(container.clone()),
                        ..Default::default()
                    })))
                    .await?;
                MessageResponse::Consumed
            }
            _ => MessageResponse::NotConsumed,
        };
        self.refresh().await?;
        Ok(result)
    }

    async fn initialise(&mut self) -> Result<()> {
        self.list_state = TableState::default();
        self.list_state.select(Some(0));

        self.refresh().await?;
        Ok(())
    }

    async fn set_visible(&mut self, cx: AppContext) -> Result<()> {
        self.visible = true;
        self.initialise()
            .await
            .context("unable to set containers as visible")?;

        // If a context has been passed in, choose that item in list
        // this ist to allo logs, attach etc to appear to revert to previous
        // state
        // I'm sure there is a more sensible way of doing this...
        if let Some(container) = cx.docker_container {
            let container_id = container.id;
            for (idx, c) in self.containers.iter().enumerate() {
                if c.id == container_id {
                    self.list_state.select(Some(idx));
                    break;
                }
            }
        }
        Ok(())
    }

    async fn set_invisible(&mut self) -> Result<()> {
        self.visible = false;
        Ok(())
    }

    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

impl Containers {
    pub async fn new(docker: Docker, tx: Sender<Message<Key, Transition>>) -> Self {
        let page_help = PageHelp::new(NAME.into())
            .add_input(format!("{}", A_KEY), "exec".into())
            .add_input(format!("{CTRL_D_KEY}"), "delete".into())
            .add_input(format!("{R_KEY}"), "run".into())
            .add_input(format!("{S_KEY}"), "stop".into())
            .add_input(format!("{G_KEY}"), "to-top".into())
            .add_input(format!("{SHIFT_G_KEY}"), "to-bottom".into())
            .add_input(format!("{L_KEY}"), "logs".into());

        Self {
            name: String::from(NAME),
            page_help: Arc::new(Mutex::new(page_help)),
            tx,
            visible: false,
            docker,
            containers: vec![],
            list_state: TableState::default(),
            modal: None,
        }
    }

    async fn refresh(&mut self) -> Result<(), color_eyre::eyre::Error> {
        self.containers = DockerContainer::list(&self.docker).await?;
        Ok(())
    }

    fn increment_list(&mut self) {
        let current_idx = self.list_state.selected();
        match current_idx {
            None => self.list_state.select(Some(0)),
            Some(current_idx) => {
                if !self.containers.is_empty() && current_idx < self.containers.len() - 1 {
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

    fn get_container(&self) -> Result<&DockerContainer> {
        if let Some(container_idx) = self.list_state.selected() {
            if let Some(container) = self.containers.get(container_idx) {
                return Ok(container);
            }
        }
        bail!("no container id found");
    }

    async fn start_container(&mut self) -> Result<Option<()>> {
        if let Ok(container) = self.get_container() {
            container.start(&self.docker).await?;
            self.refresh().await?;
            return Ok(Some(()));
        }
        Ok(None)
    }

    async fn stop_container(&mut self) -> Result<Option<()>> {
        if let Ok(container) = self.get_container() {
            container.stop(&self.docker).await?;

            self.refresh().await?;
            return Ok(Some(()));
        }
        Ok(None)
    }

    fn delete_container(&mut self) -> Result<()> {
        if let Ok(container) = self.get_container() {
            let name = container.names.clone();
            let image = container.image.clone();

            let message = match container.running {
                true => {
                    format!("Are you sure you wish to delete container {name} (image = {image})?  This container is currently running; this will result in a force deletion.")
                }
                false => {
                    format!("Are you sure you wish to delete container {name} (image = {image})?")
                }
            };

            let cb = Arc::new(FutureMutex::new(DeleteContainer::new(
                self.docker.clone(),
                container.clone(),
                container.running,
            )));

            let mut modal =
                BooleanModal::<ModalTypes>::new("Delete".into(), ModalTypes::DeleteContainer);
            modal.initialise(message, Some(cb));
            self.modal = Some(modal);
        } else {
            bail!("Ahhh")
        }
        Ok(())
    }
}

impl Component for Containers {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let rows = self.containers.clone().into_iter().map(|c| {
            let style = match c.running {
                true => Style::default().fg(Color::Green),
                false => Style::default(),
            };

            Row::new(vec![
                c.id, c.image, c.command, c.created, c.status, c.ports, c.names,
            ])
            .style(style)
        });
        let columns = Row::new(vec![
            "ID", "Image", "Command", "Created", "Status", "Ports", "Names",
        ]);

        let widths = [
            Constraint::Percentage(12),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(13),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ];

        let table = Table::new(rows.clone(), widths)
            .header(columns.clone().style(Style::new().bold()))
            .highlight_style(Style::new().reversed());

        f.render_stateful_widget(table, area, &mut self.list_state);

        if let Some(m) = self.modal.as_mut() {
            if let ModalState::Open(_) = m.state {
                m.draw(f, area)
            }
        }
    }
}

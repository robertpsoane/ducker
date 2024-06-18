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
        confirmation_modal::{ConfirmationModal, ModalState},
        help::PageHelp,
    },
    docker::container::{self, DockerContainer},
    events::{message::MessageResponse, Key, Message, Transition},
    state::CurrentPage,
    traits::{Component, Page},
};

const NAME: &str = "Containers";

const UP_KEY: Key = Key::Up;
const DOWN_KEY: Key = Key::Down;

const _A_KEY: Key = Key::Char('a');
const J_KEY: Key = Key::Char('j');
const K_KEY: Key = Key::Char('k');
const D_KEY: Key = Key::Char('d');
const R_KEY: Key = Key::Char('r');
const S_KEY: Key = Key::Char('s');
const G_KEY: Key = Key::Char('g');
const L_KEY: Key = Key::Char('l');
const SHIFT_G_KEY: Key = Key::Char('G');

#[derive(Debug)]
pub struct Containers {
    pub name: String,
    pub visible: bool,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
    docker: Docker,
    containers: Vec<DockerContainer>,
    list_state: TableState,
    delete_modal: ConfirmationModal<bool>,
}

#[async_trait::async_trait]
impl Page for Containers {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        if !self.visible {
            return Ok(MessageResponse::NotConsumed);
        }

        // If the delete modal is open, we process it; if it is open or complete, and the
        // result is Consumed, we exit early with the Consumed result
        let delete_modal_state = self.delete_modal.state.clone();
        if let ModalState::Open(_) = delete_modal_state {
            let delete_modal_res = self.delete_modal.update(message).await?;
            if delete_modal_res == MessageResponse::Consumed {
                return Ok(delete_modal_res);
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
            D_KEY => match self.delete_container() {
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
            L_KEY => {
                let container = self.get_container()?;
                self.tx
                    .send(Message::Transition(Transition::ToLogPage(
                        container.clone(),
                    )))
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

    async fn set_visible(&mut self, _: CurrentPage) -> Result<()> {
        self.visible = true;
        self.initialise()
            .await
            .context("unable to set containers as visible")?;
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
    pub async fn new(docker: Docker, tx: Sender<Message<Key, Transition>>) -> Result<Self> {
        let page_help = PageHelp::new(NAME.into())
            // .add_input(format!("{}", A_KEY), "attach".into())
            .add_input(format!("{D_KEY}"), "delete".into())
            .add_input(format!("{R_KEY}"), "run".into())
            .add_input(format!("{S_KEY}"), "stop".into())
            .add_input(format!("{G_KEY}"), "to-top".into())
            .add_input(format!("{SHIFT_G_KEY}"), "to-bottom".into())
            .add_input(format!("{L_KEY}"), "logs".into());

        Ok(Self {
            name: String::from(NAME),
            page_help: Arc::new(Mutex::new(page_help)),
            tx,
            visible: false,
            docker,
            containers: vec![],
            list_state: TableState::default(),
            delete_modal: ConfirmationModal::<bool>::new("Delete".into()),
        })
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
                    format!("Are you sure you wish to delete container {name} (image {image})?  This container is currently running; this will result in a force deletion.")
                }
                false => {
                    format!("Are you sure you wish to delete container {name} (image {image})?")
                }
            };

            let cb = Arc::new(FutureMutex::new(DeleteContainer::new(
                self.docker.clone(),
                container.clone(),
                container.running,
            )));
            self.delete_modal.initialise(message, cb);
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

        match self.delete_modal.state {
            ModalState::Open(_) => self.delete_modal.draw(f, area),
            _ => {}
        }
    }
}

use std::{
    default,
    time::{Duration, UNIX_EPOCH},
};

use bollard::{container::ListContainersOptions, secret::ContainerSummary, Docker};
use chrono::prelude::DateTime;
use chrono::Local;
use color_eyre::{
    config,
    eyre::{bail, Context, Result},
};
use ratatui::{
    layout::Rect,
    prelude::*,
    style::Style,
    widgets::{Row, Table, TableState},
    Frame,
};

use crate::{
    component::Component,
    events::{message::MessageResponse, Key},
};

use super::confirmation_modal::{BooleanOptions, ConfirmationModal, ModalState};

const NAME: &str = "Containers";

#[derive(Debug)]
pub struct Containers {
    pub name: String,
    pub visible: bool,
    docker: Docker,
    containers: Vec<ContainerSummary>,
    list_state: TableState,
    delete_modal: ConfirmationModal<BooleanOptions>,
}

impl Containers {
    pub async fn new(visible: bool, docker: Docker) -> Result<Self> {
        let mut instance = Self {
            name: String::from(NAME),
            visible,
            docker,
            containers: vec![],
            list_state: TableState::default(),
            delete_modal: ConfirmationModal::<BooleanOptions>::new("Delete".into()),
        };

        if instance.visible {
            instance
                .set_visible()
                .await
                .context("attempt to set new containers list as visible failed")?;
        }

        Ok(instance)
    }

    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        if !self.visible {
            return Ok(MessageResponse::NotConsumed);
        }

        // TODO: The validator should take a callback on initialisation that manages the delete
        // or on instantiation with extra variables passed on on init - probabyl
        // makes more sense on init
        //
        // Then the ModalState should have Open(message) and Closed, Complete;
        // when closed, it is in effect dead.
        //
        // In the ModalState::Closed, it is possible that the modal can become
        // Open, however once it becomes Open, it will stay open until it migrates to
        // Closed
        // When it is Open the branch should check if the state has changed to Complete, at
        // which point it should be reset from outside
        // The Complete branch should also close the modal from outside
        //
        // Potentially there is still value in keeping the generic type for the modal state
        // to allow multiple implementations for different types, but to a certain extent
        // that is abusing generics to create some sort of inheritance type structure
        let delete_modal_state = self.delete_modal.state.clone();
        let result = match delete_modal_state {
            ModalState::Invisible => match message {
                Key::Up | Key::Char('k') => {
                    self.decrement_list();
                    MessageResponse::Consumed
                }
                Key::Down | Key::Char('j') => {
                    self.increment_list();
                    MessageResponse::Consumed
                }
                Key::Char('d') => {
                    if let Ok(container) = self.get_container() {
                        let container_id = container.id.clone().unwrap_or_default();
                        let image = container.image.clone().unwrap_or_default();
                        self.delete_modal.initialise(format!(
                            "Are you sure you wish to delete container {container_id}, running {image}?"
                        ));
                        MessageResponse::Consumed
                    } else {
                        MessageResponse::NotConsumed
                    }
                }
                Key::Char('r') => {
                    self.start_container()
                        .await
                        .context("could not start container")?;
                    MessageResponse::NotConsumed
                }
                Key::Char('s') => {
                    self.stop_container()
                        .await
                        .context("could not start container")?;
                    MessageResponse::NotConsumed
                }
                _ => MessageResponse::NotConsumed,
            },
            ModalState::Waiting(_) => {
                let update_res = self.delete_modal.update(message).await?;
                if update_res == MessageResponse::Consumed {
                    if let ModalState::Complete(res) = self.delete_modal.state.clone() {
                        match res {
                            BooleanOptions::Yes => {
                                self.delete_container()
                                    .await
                                    .context("could not delete current container")?;
                                self.delete_modal.reset();
                            }
                            BooleanOptions::No => self.delete_modal.reset(),
                        }
                    }
                }
                update_res
            }
            ModalState::Complete(_) => {
                self.delete_modal.reset();
                MessageResponse::NotConsumed
            }
        };
        Ok(result)
    }

    pub async fn initialise_state(&mut self) -> Result<()> {
        self.list_state = TableState::default();
        self.list_state.select(Some(0));

        self.refresh().await?;
        Ok(())
    }

    async fn refresh(&mut self) -> Result<(), color_eyre::eyre::Error> {
        self.containers = self
            .docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
            .context("unable to retrieve list of containers")?;
        Ok(())
    }

    pub async fn set_visible(&mut self) -> Result<()> {
        self.visible = true;
        self.initialise_state()
            .await
            .context("unable to set containers as visible")?;
        Ok(())
    }

    pub async fn set_invisible(&mut self) {
        self.visible = false
    }

    fn increment_list(&mut self) {
        let current_idx = self.list_state.selected();
        match current_idx {
            None => self.list_state.select(Some(0)),
            Some(current_idx) => {
                if self.containers.len() != 0 && current_idx < self.containers.len() - 1 {
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

    fn get_container(&self) -> Result<&ContainerSummary> {
        if let Some(container_idx) = self.list_state.selected() {
            if let Some(container) = self.containers.get(container_idx) {
                return Ok(container);
            }
        }
        bail!("no container id found");
    }

    async fn delete_container(&mut self) -> Result<Option<()>> {
        if let Ok(container) = self.get_container() {
            if let Some(container_id) = container.id.clone() {
                self.docker
                    .remove_container(&container_id, None)
                    .await
                    .unwrap();
            }

            self.refresh().await?;
            return Ok(Some(()));
        }
        Ok(None)
    }

    async fn start_container(&mut self) -> Result<Option<()>> {
        if let Ok(container) = self.get_container() {
            if let Some(container_id) = container.id.clone() {
                self.docker
                    .start_container::<String>(&container_id, None)
                    .await
                    .context("failed to start container")?;
            }

            self.refresh().await?;
            return Ok(Some(()));
        }
        Ok(None)
    }

    async fn stop_container(&mut self) -> Result<Option<()>> {
        if let Ok(container) = self.get_container() {
            if let Some(container_id) = container.id.clone() {
                self.docker
                    .stop_container(&container_id, None)
                    .await
                    .context("failed to start container")?;
            }

            self.refresh().await?;
            return Ok(Some(()));
        }
        Ok(None)
    }
}

impl Component for Containers {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let rows = get_container_rows(&self.containers);
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
            ModalState::Waiting(_) => self.delete_modal.draw(f, area),
            _ => {}
        }
    }
}

fn get_container_rows(containers: &[ContainerSummary]) -> Vec<Row> {
    let rows = containers
        .iter()
        .map(|c| {
            let ports = match c.ports.clone() {
                Some(p) => p
                    .into_iter()
                    .map(|p| {
                        let ip = p.ip.unwrap_or_default();
                        let private_port = p.private_port.to_string();
                        let public_port = match p.public_port {
                            Some(port) => port.to_string(),
                            None => String::new(),
                        };
                        let typ = match p.typ {
                            Some(t) => format!("{:?}", t),
                            None => String::new(),
                        };

                        format!("{}:{}:{}:{}", ip, private_port, public_port, typ)
                    })
                    .collect::<Vec<String>>()
                    .join(", "),
                None => "".into(),
            };

            let datetime = DateTime::<Local>::from(
                UNIX_EPOCH
                    + Duration::from_secs(
                        c.created.unwrap_or_default().try_into().unwrap_or_default(),
                    ),
            )
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

            let style = match c.state.clone().unwrap_or_default().as_str() {
                "running" => Style::default().fg(Color::Green),
                _ => Style::default(),
            };

            Row::new(vec![
                c.id.clone().unwrap_or_default(),
                c.image.clone().unwrap_or_default(),
                c.command.clone().unwrap_or_default(),
                datetime,
                c.status.clone().unwrap_or_default(),
                ports,
                c.names.clone().unwrap_or_default().join(", "),
            ])
            .style(style)
        })
        .collect::<Vec<Row>>();
    rows
}

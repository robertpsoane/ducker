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

use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::Sender;

use crate::{
    callbacks::DeleteContainer,
    components::{
        boolean_modal::{BooleanModal, ModalState},
        help::{PageHelp, PageHelpBuilder},
    },
    config::Config,
    context::AppContext,
    docker::container::DockerContainer,
    events::{message::MessageResponse, Key, Message, Transition},
    sorting::{
        sort_containers_by_created, sort_containers_by_image, sort_containers_by_name,
        sort_containers_by_ports, sort_containers_by_status, ContainerSortField, SortOrder,
        SortState,
    },
    traits::{Close, Component, ModalComponent, Page},
};

const NAME: &str = "Containers";

const UP_KEY: Key = Key::Up;
const DOWN_KEY: Key = Key::Down;

const A_KEY: Key = Key::Char('a');
const J_KEY: Key = Key::Char('j');
const K_KEY: Key = Key::Char('k');
const CTRL_D_KEY: Key = Key::Ctrl('d');
const D_KEY: Key = Key::Char('d');
const R_KEY: Key = Key::Char('r');
const S_KEY: Key = Key::Char('s');
const G_KEY: Key = Key::Char('g');
const L_KEY: Key = Key::Char('l');
const SHIFT_G_KEY: Key = Key::Char('G');

// Sorting keys
const SHIFT_N_KEY: Key = Key::Char('N');
const SHIFT_I_KEY: Key = Key::Char('I');
const SHIFT_S_KEY: Key = Key::Char('S');
const SHIFT_C_KEY: Key = Key::Char('C');
const SHIFT_P_KEY: Key = Key::Char('P');

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalTypes {
    DeleteContainer,
}

#[derive(Debug)]
pub struct Containers {
    config: Arc<Config>,
    pub name: String,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
    docker: Docker,
    containers: Vec<DockerContainer>,
    list_state: TableState,
    modal: Option<BooleanModal<ModalTypes>>,
    stopping_containers: Arc<Mutex<HashSet<String>>>,
    sort_state: SortState<ContainerSortField>,
    container_fields: Vec<String>,
}

#[async_trait::async_trait]
impl Page for Containers {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
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
                self.tx
                    .send(Message::Transition(Transition::ToAttach(
                        self.get_context()?,
                    )))
                    .await?;
                MessageResponse::Consumed
            }
            L_KEY => {
                self.tx
                    .send(Message::Transition(Transition::ToLogPage(
                        self.get_context()?,
                    )))
                    .await?;
                MessageResponse::Consumed
            }
            D_KEY => {
                self.tx
                    .send(Message::Transition(Transition::ToDescribeContainerPage(
                        self.get_context()?,
                    )))
                    .await?;
                MessageResponse::Consumed
            }
            // Sorting functionality
            SHIFT_N_KEY => {
                self.sort_state.toggle_or_set(ContainerSortField::Name);
                self.sort_containers();
                MessageResponse::Consumed
            }
            SHIFT_I_KEY => {
                self.sort_state.toggle_or_set(ContainerSortField::Image);
                self.sort_containers();
                MessageResponse::Consumed
            }
            SHIFT_S_KEY => {
                self.sort_state.toggle_or_set(ContainerSortField::Status);
                self.sort_containers();
                MessageResponse::Consumed
            }
            SHIFT_C_KEY => {
                self.sort_state.toggle_or_set(ContainerSortField::Created);
                self.sort_containers();
                MessageResponse::Consumed
            }
            SHIFT_P_KEY => {
                self.sort_state.toggle_or_set(ContainerSortField::Ports);
                self.sort_containers();
                MessageResponse::Consumed
            }
            _ => MessageResponse::NotConsumed,
        };
        self.refresh().await?;
        Ok(result)
    }

    async fn initialise(&mut self, cx: AppContext) -> Result<()> {
        self.container_fields = super::super::config::parse_format_fields(self.config.format.as_deref().unwrap_or(""));

        self.list_state = TableState::default();
        self.list_state.select(Some(0));

        self.refresh()
            .await
            .context("unable to set refresh containers")?;

        // Apply initial sorting
        self.sort_containers();

        // If a context has been passed in, choose that item in list
        // this ist to allo logs, attach etc to appear to revert to previous
        // state
        // I'm sure there is a more sensible way of doing this...
        let container_id: String;
        if let Some(container) = cx.docker_container {
            container_id = container.id;
        } else if let Some(thing) = cx.describable {
            container_id = thing.get_id();
        } else {
            return Ok(());
        }

        for (idx, c) in self.containers.iter().enumerate() {
            if c.id == container_id {
                self.list_state.select(Some(idx));
                break;
            }
        }

        Ok(())
    }

    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

#[async_trait::async_trait]
impl Close for Containers {}

impl Containers {
    pub fn new(docker: Docker, tx: Sender<Message<Key, Transition>>, config: Arc<Config>) -> Self {
        let page_help = PageHelpBuilder::new(NAME.to_string(), config.clone())
            .add_input(format!("{A_KEY}"), "exec".to_string())
            .add_input(format!("{CTRL_D_KEY}"), "delete".to_string())
            .add_input(format!("{R_KEY}"), "run".to_string())
            .add_input(format!("{S_KEY}"), "stop".to_string())
            .add_input(format!("{G_KEY}"), "top".to_string())
            .add_input(format!("{SHIFT_G_KEY}"), "bottom".to_string())
            .add_input(format!("{L_KEY}"), "logs".to_string())
            .build();

        Self {
            config,
            name: String::from(NAME),
            page_help: Arc::new(Mutex::new(page_help)),
            tx,
            docker,
            containers: vec![],
            list_state: TableState::default(),
            modal: None,
            stopping_containers: Arc::new(Mutex::new(HashSet::new())),
            sort_state: SortState::new(ContainerSortField::Name),
            container_fields: vec![],
        }
    }

    async fn refresh(&mut self) -> Result<(), color_eyre::eyre::Error> {
        self.containers = DockerContainer::list(&self.docker).await?;
        self.sort_containers();
        Ok(())
    }

    fn sort_containers(&mut self) {
        let field = self.sort_state.field;
        let order = self.sort_state.order;

        self.containers.sort_by(|a, b| match field {
            ContainerSortField::Name => sort_containers_by_name(a, b, order),
            ContainerSortField::Image => sort_containers_by_image(a, b, order),
            ContainerSortField::Status => sort_containers_by_status(a, b, order),
            ContainerSortField::Created => sort_containers_by_created(a, b, order),
            ContainerSortField::Ports => sort_containers_by_ports(a, b, order),
        });
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
            self.stopping_containers
                .lock()
                .unwrap()
                .insert(container.id.clone());

            let c = container.clone();
            let docker = self.docker.clone();
            let tx = self.tx.clone();
            let stopping_containers = self.stopping_containers.clone();
            tokio::spawn(async move {
                let message = if c.stop(&docker).await.is_ok() {
                    Message::Tick
                } else {
                    let msg = format!("Failed to delete container {}", c.id);
                    Message::Error(msg)
                };
                stopping_containers.lock().unwrap().remove(&c.id);
                let _ = tx.send(message).await;
            });

            // Second spawned taskt is used to update the state

            self.refresh().await?;
            return Ok(Some(()));
        }
        Ok(None)
    }

    fn delete_container(&mut self) -> Result<()> {
        if let Ok(container) = self.get_container() {
            let name = container.names.clone();
            let image = container.image.clone();

            let message = if container.running {
                format!("Are you sure you wish to delete container {name} (image = {image})?  This container is currently running; this will result in a force deletion.")
            } else {
                format!("Are you sure you wish to delete container {name} (image = {image})?")
            };

            let cb = Arc::new(FutureMutex::new(DeleteContainer::new(
                self.docker.clone(),
                container.clone(),
                container.running,
                self.tx.clone(),
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

    fn get_context(&self) -> Result<AppContext> {
        let container = self.get_container()?;

        let then = Some(Box::new(Transition::ToContainerPage(AppContext {
            docker_container: Some(container.clone()),
            ..Default::default()
        })));

        let cx = AppContext {
            describable: Some(Box::new(container.clone())),
            then,
            docker_container: Some(container.clone()),
            ..Default::default()
        };

        Ok(cx)
    }

    fn field_to_header(&self, field: &str) -> String {
        match field.to_lowercase().as_str() {
            "id" => "ID".to_string(),
            "image" => "Image".to_string(),
            "command" => "Command".to_string(),
            "created" => "Created".to_string(),
            "status" => "Status".to_string(),
            "ports" => "Ports".to_string(),
            "names" => "Names".to_string(),
            _ => field.to_string(),
        }
    }

    fn field_to_sort_field(&self, field: &str) -> Option<ContainerSortField> {
        match field.to_lowercase().as_str() {
            "name" | "names" => Some(ContainerSortField::Name),
            "image" => Some(ContainerSortField::Image),
            "status" => Some(ContainerSortField::Status),
            "created" => Some(ContainerSortField::Created),
            "ports" => Some(ContainerSortField::Ports),
            _ => None,
        }
    }

}

impl Component for Containers {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let rows: Vec<Row> = self.containers.clone().into_iter().map(|c| {
            let style = if self.stopping_containers.lock().unwrap().contains(&c.id) {
                Style::default().fg(self.config.theme.negative_highlight())
            } else if c.running {
                Style::default().fg(self.config.theme.positive_highlight())
            } else {
                Style::default()
            };

            let row_vec: Vec<String> = self.container_fields.iter().map(|f| c.get_field(f)).collect();
            Row::new(row_vec).style(style)
        }).collect();

        // Create column headers with sort indicators
        let columns_strings: Vec<String> = self.container_fields.iter().map(|f| {
            let mut hdr = self.field_to_header(f);
            if let Some(sf) = self.field_to_sort_field(f) {
                if self.sort_state.field == sf {
                    let arrow = match self.sort_state.order {
                        SortOrder::Ascending => " ↑",
                        SortOrder::Descending => " ↓",
                    };
                    hdr += arrow;
                }
            }
            hdr
        }).collect();
        let columns = Row::new(columns_strings).style(Style::new().bold());

        let widths: Vec<Constraint> = match self.container_fields.len() {
            1 => vec![Constraint::Percentage(100)],
            2 => vec![Constraint::Percentage(30), Constraint::Percentage(70)],
            _ if self.container_fields.len() >= 3 => {
                let mut constraints = vec![Constraint::Length(12)]; // ID column
                let remaining = self.container_fields.len() - 1;
                let each_remaining = 88 / remaining as u16;
                for _ in 0..remaining {
                    constraints.push(Constraint::Percentage(each_remaining));
                }
                constraints
            },
            _ => vec![Constraint::Percentage(100 / self.container_fields.len() as u16); self.container_fields.len()],
        };

        let table = Table::new(rows, widths)
            .header(columns)
            .row_highlight_style(Style::new().reversed());

        f.render_stateful_widget(table, area, &mut self.list_state);

        if let Some(m) = self.modal.as_mut() {
            if let ModalState::Open(_) = m.state {
                m.draw(f, area)
            }
        }
    }
}

use bollard::Docker;
use color_eyre::eyre::{bail, Context, ContextCompat, Result};
use futures::lock::Mutex as FutureMutex;
use ratatui::{
    layout::Rect,
    prelude::*,
    style::Style,
    widgets::{Row, Table, TableState},
    Frame,
};
use ratatui_macros::constraints;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::Sender;

use crate::{
    // callbacks::delete_network::DeleteNetwork,
    callbacks::{delete_network::DeleteNetwork, empty_callable::EmptyCallable},
    components::{
        boolean_modal::{BooleanModal, ModalState},
        help::{PageHelp, PageHelpBuilder},
    },
    config::Config,
    context::AppContext,
    docker::network::DockerNetwork,
    events::{message::MessageResponse, Key, Message, Transition},
    traits::{Close, Component, ModalComponent, Page},
};

const NAME: &str = "Networks";

const UP_KEY: Key = Key::Up;
const DOWN_KEY: Key = Key::Down;

const J_KEY: Key = Key::Char('j');
const K_KEY: Key = Key::Char('k');
const CTRL_D_KEY: Key = Key::Ctrl('d');
const SHIFT_D_KEY: Key = Key::Char('D');
const D_KEY: Key = Key::Char('d');
const G_KEY: Key = Key::Char('g');
const SHIFT_G_KEY: Key = Key::Char('G');

#[derive(Debug)]
enum ModalTypes {
    DeleteNetwork,
    FailedToDeleteNetwork,
}

#[derive(Debug)]
pub struct Network {
    pub name: String,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
    docker: Docker,
    networks: Vec<DockerNetwork>,
    list_state: TableState,
    modal: Option<BooleanModal<ModalTypes>>,
    show_dangling: bool,
}

#[async_trait::async_trait]
impl Page for Network {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        self.refresh().await?;

        let res = self.update_modal(message).await?;
        if res == MessageResponse::Consumed {
            return Ok(res);
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
            SHIFT_D_KEY => {
                self.show_dangling = !self.show_dangling;
                MessageResponse::Consumed
            }
            G_KEY => {
                self.list_state.select(Some(0));
                MessageResponse::Consumed
            }
            SHIFT_G_KEY => {
                self.list_state.select(Some(self.networks.len() - 1));
                MessageResponse::Consumed
            }
            CTRL_D_KEY => match self.delete_network() {
                Ok(()) => MessageResponse::Consumed,
                Err(_) => MessageResponse::NotConsumed,
            },
            D_KEY => {
                self.tx
                    .send(Message::Transition(Transition::ToDescribeContainerPage(
                        self.get_context()?,
                    )))
                    .await?;
                MessageResponse::Consumed
            }
            _ => MessageResponse::NotConsumed,
        };
        Ok(result)
    }

    async fn initialise(&mut self, cx: AppContext) -> Result<()> {
        self.list_state = TableState::default();
        self.list_state.select(Some(0));

        self.refresh().await.context("unable to refresh networks")?;

        let network_id: String;
        if let Some(network) = cx.docker_network {
            network_id = network.name;
        } else if let Some(thing) = cx.describable {
            network_id = thing.get_id();
        } else {
            return Ok(());
        }

        for (idx, c) in self.networks.iter().enumerate() {
            if c.name == network_id {
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
impl Close for Network {}

impl Network {
    #[must_use]
    pub fn new(docker: Docker, tx: Sender<Message<Key, Transition>>, config: Box<Config>) -> Self {
        let page_help = PageHelpBuilder::new(NAME.into(), config.clone())
            .add_input(format!("{CTRL_D_KEY}"), "delete".into())
            .add_input(format!("{G_KEY}"), "top".into())
            .add_input(format!("{SHIFT_G_KEY}"), "bottom".into())
            // .add_input(format!("{SHIFT_D_KEY}"), "dangling".into())
            .add_input(format!("{D_KEY}"), "describe".into())
            .build();

        Self {
            name: String::from(NAME),
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
            docker,
            networks: vec![],
            list_state: TableState::default(),
            modal: None,
            show_dangling: false,
        }
    }

    async fn refresh(&mut self) -> Result<(), color_eyre::eyre::Error> {
        let mut filters: HashMap<String, Vec<String>> = HashMap::new();
        filters.insert("dangling".into(), vec!["false".into()]);

        self.networks = DockerNetwork::list(&self.docker)
            .await
            .context("unable to retrieve list of networks")?;
        Ok(())
    }

    async fn update_modal(&mut self, message: Key) -> Result<MessageResponse> {
        // Due to the fact only 1 thing should be operating at a time, we can do this to reduce unnecessary nesting
        if self.modal.is_none() {
            return Ok(MessageResponse::NotConsumed);
        }
        let m = self.modal.as_mut().context(
            "a modal magically vanished between the check that it exists and the operation on it",
        )?;

        if let ModalState::Open(_) = m.state {
            match m.update(message).await {
                Ok(_) => {
                    if let ModalState::Closed = m.state {
                        self.modal = None;
                    }
                }
                Err(e) => {
                    if let ModalTypes::DeleteNetwork = m.discriminator {
                        let msg =
                            "An error occurred deleting this network.  It is likely still in use.  Will not try again.";
                        let mut modal = BooleanModal::<ModalTypes>::new(
                            "Failed Deletion".into(),
                            ModalTypes::FailedToDeleteNetwork,
                        );

                        modal.initialise(
                            msg.into(),
                            Some(Arc::new(FutureMutex::new(EmptyCallable::new()))),
                        );
                        self.modal = Some(modal)
                    } else {
                        return Err(e);
                    }
                }
            }
            Ok(MessageResponse::Consumed)
        } else {
            Ok(MessageResponse::NotConsumed)
        }
    }

    fn increment_list(&mut self) {
        let current_idx = self.list_state.selected();
        match current_idx {
            None => self.list_state.select(Some(0)),
            Some(current_idx) => {
                if !self.networks.is_empty() && current_idx < self.networks.len() - 1 {
                    self.list_state.select(Some(current_idx + 1));
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
                    self.list_state.select(Some(current_idx - 1));
                }
            }
        }
    }

    fn get_network(&self) -> Result<&DockerNetwork> {
        if let Some(network_idx) = self.list_state.selected() {
            if let Some(network) = self.networks.get(network_idx) {
                return Ok(network);
            }
        }
        bail!("no container id found");
    }

    fn get_context(&self) -> Result<AppContext> {
        let network = self.get_network()?;

        let then = Some(Box::new(Transition::ToNetworkPage(AppContext {
            docker_network: Some(network.clone()),
            ..Default::default()
        })));

        let cx = AppContext {
            describable: Some(Box::new(network.clone())),
            then,
            ..Default::default()
        };

        Ok(cx)
    }

    fn delete_network(&mut self) -> Result<()> {
        if let Ok(network) = self.get_network() {
            let name = network.name.clone();

            let cb = Arc::new(FutureMutex::new(DeleteNetwork::new(
                self.docker.clone(),
                network.clone(),
            )));

            let mut modal =
                BooleanModal::<ModalTypes>::new("Delete".into(), ModalTypes::DeleteNetwork);

            modal.initialise(
                format!("Are you sure you wish to delete network {name})?"),
                Some(cb),
            );
            self.modal = Some(modal);
        } else {
            bail!("failed to setup deletion modal")
        }
        Ok(())
    }
}

impl Component for Network {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let rows = get_network_rows(&self.networks);
        let columns = Row::new(vec!["Id", "Name", "Driver", "Created", "Scope"]);

        let widths = constraints![==30%, ==25%, ==15%, ==15%, ==15%];

        let table = Table::new(rows.clone(), widths)
            .header(columns.clone().style(Style::new().bold()))
            .highlight_style(Style::new().reversed());

        f.render_stateful_widget(table, area, &mut self.list_state);

        if let Some(m) = self.modal.as_mut() {
            if let ModalState::Open(_) = m.state {
                m.draw(f, area);
            }
        }
    }
}

fn get_network_rows(networks: &[DockerNetwork]) -> Vec<Row> {
    let rows = networks
        .iter()
        .map(|c| {
            Row::new(vec![
                c.id.clone(),
                c.name.clone(),
                c.driver.clone(),
                c.created_at.clone(),
                c.scope.clone(),
            ])
        })
        .collect::<Vec<Row>>();
    rows
}

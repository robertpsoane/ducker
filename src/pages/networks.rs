use bollard::Docker;
use color_eyre::eyre::{Context, ContextCompat, Result, bail};
use futures::lock::Mutex as FutureMutex;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    widgets::{Row, Table, TableState},
};
use ratatui_macros::constraints;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;

use crate::{
    // callbacks::delete_network::DeleteNetwork,
    callbacks::{
        delete_network::DeleteNetwork, empty_callable::EmptyCallable, prune_networks::PruneNetworks,
    },
    components::{
        boolean_modal::{BooleanModal, ModalState},
        help::{PageHelp, PageHelpBuilder},
        text_input_wrapper::TextInputWrapper,
    },
    config::Config,
    context::AppContext,
    docker::network::DockerNetwork,
    events::{Key, Message, Transition, message::MessageResponse},
    sorting::{
        NetworkSortField, SortOrder, SortState, sort_networks_by_created, sort_networks_by_driver,
        sort_networks_by_id, sort_networks_by_name, sort_networks_by_scope,
    },
    traits::{Close, Component, ModalComponent, Page},
};

const NAME: &str = "Networks";

const UP_KEY: Key = Key::Up;
const DOWN_KEY: Key = Key::Down;
const PAGE_UP_KEY: Key = Key::PageUp;
const PAGE_DOWN_KEY: Key = Key::PageDown;

const J_KEY: Key = Key::Char('j');
const K_KEY: Key = Key::Char('k');
const CTRL_D_KEY: Key = Key::Ctrl('d');
const SHIFT_D_KEY: Key = Key::Char('D');
const D_KEY: Key = Key::Char('d');
const G_KEY: Key = Key::Char('g');
const SHIFT_G_KEY: Key = Key::Char('G');
const CTRL_P_KEY: Key = Key::Ctrl('p');
const SLASH_KEY: Key = Key::Char('/');
const ESC_KEY: Key = Key::Esc;
const ENTER_KEY: Key = Key::Enter;

// Sort keys
const SHIFT_N_KEY: Key = Key::Char('N');
const SHIFT_C_KEY: Key = Key::Char('C');
const SHIFT_S_KEY: Key = Key::Char('S');

type NetworkSortState = SortState<NetworkSortField>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    filtered_networks: Vec<DockerNetwork>,
    list_state: TableState,
    modal: Option<BooleanModal<ModalTypes>>,
    sort_state: NetworkSortState,
    table_height: u16,
    is_filtering: bool,
    filter_input: TextInputWrapper,
}

#[async_trait::async_trait]
impl Page for Network {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        self.refresh().await?;

        let res = self.update_modal(message).await?;
        if res == MessageResponse::Consumed {
            return Ok(res);
        }

        if self.is_filtering {
            match message {
                ESC_KEY => {
                    self.is_filtering = false;
                    self.filter_input.reset();
                    self.filter_networks(true);
                    self.sort_networks();
                    return Ok(MessageResponse::Consumed);
                }
                ENTER_KEY => {
                    self.is_filtering = false;
                    return Ok(MessageResponse::Consumed);
                }
                _ => {
                    self.filter_input.update(message)?;
                    self.filter_networks(true);
                    self.sort_networks();
                    return Ok(MessageResponse::Consumed);
                }
            }
        }

        let result = match message {
            SLASH_KEY => {
                self.is_filtering = true;
                MessageResponse::Consumed
            }
            UP_KEY | K_KEY => {
                self.scroll_up(1);
                MessageResponse::Consumed
            }
            PAGE_UP_KEY => {
                self.scroll_up(self.table_height.into());
                MessageResponse::Consumed
            }
            DOWN_KEY | J_KEY => {
                self.scroll_down(1);
                MessageResponse::Consumed
            }
            PAGE_DOWN_KEY => {
                self.scroll_down(self.table_height.into());
                MessageResponse::Consumed
            }
            SHIFT_D_KEY => {
                self.sort_state.toggle_or_set(NetworkSortField::Driver);
                self.sort_networks();
                MessageResponse::Consumed
            }
            G_KEY => {
                self.list_state.select(Some(0));
                MessageResponse::Consumed
            }
            SHIFT_G_KEY => {
                self.list_state.select(Some(self.filtered_networks.len() - 1));
                MessageResponse::Consumed
            }
            SHIFT_N_KEY => {
                self.sort_state.toggle_or_set(NetworkSortField::Name);
                self.sort_networks();
                MessageResponse::Consumed
            }
            SHIFT_C_KEY => {
                self.sort_state.toggle_or_set(NetworkSortField::Created);
                self.sort_networks();
                MessageResponse::Consumed
            }
            SHIFT_S_KEY => {
                self.sort_state.toggle_or_set(NetworkSortField::Scope);
                self.sort_networks();
                MessageResponse::Consumed
            }
            CTRL_D_KEY => match self.delete_network() {
                Ok(()) => MessageResponse::Consumed,
                Err(_) => MessageResponse::NotConsumed,
            },
            CTRL_P_KEY => match self.prune_networks() {
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

        for (idx, c) in self.filtered_networks.iter().enumerate() {
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
    pub fn new(docker: Docker, tx: Sender<Message<Key, Transition>>, config: Arc<Config>) -> Self {
        let page_help = PageHelpBuilder::new(NAME.to_string(), config.clone())
            .add_input(format!("{CTRL_D_KEY}"), "delete".to_string())
            .add_input(format!("{CTRL_P_KEY}"), "prune".to_string())
            .add_input(format!("{G_KEY}"), "top".to_string())
            .add_input(format!("{SHIFT_G_KEY}"), "bottom".to_string())
            .add_input(format!("{D_KEY}"), "describe".to_string())
            .build();

        Self {
            name: String::from(NAME),
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
            docker,
            networks: vec![],
            filtered_networks: vec![],
            list_state: TableState::default(),
            modal: None,
            sort_state: NetworkSortState::new(NetworkSortField::Name),
            table_height: 0,
            is_filtering: false,
            filter_input: TextInputWrapper::new("Filter".to_string(), None),
        }
    }

    async fn refresh(&mut self) -> Result<(), color_eyre::eyre::Error> {
        self.networks = DockerNetwork::list(&self.docker)
            .await
            .context("unable to retrieve list of networks")?;

        let selected_id = self.get_network().map(|c| c.id.clone()).ok();
        self.filter_networks(false);
        self.sort_networks();

        if let Some(id) = selected_id {
            if let Some(idx) = self.filtered_networks.iter().position(|c| c.id == id) {
                self.list_state.select(Some(idx));
            }
        }

        Ok(())
    }

    fn filter_networks(&mut self, reset_selection: bool) {
        let filter_text = self.filter_input.get_value().to_lowercase();
        self.filtered_networks = self
            .networks
            .iter()
            .filter(|c| {
                if filter_text.is_empty() {
                    return true;
                }
                c.id.to_lowercase().contains(&filter_text)
                    || c.name.to_lowercase().contains(&filter_text)
                    || c.driver.to_lowercase().contains(&filter_text)
            })
            .cloned()
            .collect();
        if reset_selection {
            self.list_state.select(Some(0));
        }
    }

    fn sort_networks(&mut self) {
        let field = self.sort_state.field;
        let order = self.sort_state.order;

        self.filtered_networks.sort_by(|a, b| match field {
            NetworkSortField::Id => sort_networks_by_id(a, b, order),
            NetworkSortField::Name => sort_networks_by_name(a, b, order),
            NetworkSortField::Driver => sort_networks_by_driver(a, b, order),
            NetworkSortField::Created => sort_networks_by_created(a, b, order),
            NetworkSortField::Scope => sort_networks_by_scope(a, b, order),
        });
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
                        let msg = "An error occurred deleting this network.  It is likely still in use.  Will not try again.";
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

    fn scroll_down(&mut self, amount: usize) {
        let current_idx = self.list_state.selected();
        match current_idx {
            None => self.list_state.select(Some(0)),
            Some(current_idx) => {
                if !self.filtered_networks.is_empty() {
                    let len = self.filtered_networks.len();
                    let new_idx = (current_idx + amount).min(len.saturating_sub(1));
                    self.list_state.select(Some(new_idx));
                }
            }
        }
    }

    fn scroll_up(&mut self, amount: usize) {
        let current_idx = self.list_state.selected();
        match current_idx {
            None => self.list_state.select(Some(0)),
            Some(current_idx) => {
                let new_idx = current_idx.saturating_sub(amount);
                self.list_state.select(Some(new_idx));
            }
        }
    }

    fn get_network(&self) -> Result<&DockerNetwork> {
        if let Some(network_idx) = self.list_state.selected()
            && let Some(network) = self.filtered_networks.get(network_idx)
        {
            return Ok(network);
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

    fn prune_networks(&mut self) -> Result<()> {
        let cb = Arc::new(FutureMutex::new(PruneNetworks::new(
            self.docker.clone(),
            self.tx.clone(),
        )));

        let mut modal = BooleanModal::<ModalTypes>::new("Prune".into(), ModalTypes::DeleteNetwork);

        modal.initialise(
            "Are you sure you wish to prune networks?".to_string(),
            Some(cb),
        );
        self.modal = Some(modal);
        Ok(())
    }
}

impl Component for Network {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        use ratatui::layout::{Constraint, Layout};

        let show_filter = self.is_filtering || !self.filter_input.get_value().is_empty();

        let table_area = if show_filter {
            let layout = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]);
            let [table_area, filter_area] = layout.areas(area);
            self.filter_input.draw(f, filter_area);
            table_area
        } else {
            area
        };

        self.table_height = table_area.height.saturating_sub(2);
        let rows = get_network_rows(&self.filtered_networks);
        let columns = Row::new(vec![
            get_header_with_sort_indicator("Id", NetworkSortField::Id, &self.sort_state),
            get_header_with_sort_indicator("Name", NetworkSortField::Name, &self.sort_state),
            get_header_with_sort_indicator("Driver", NetworkSortField::Driver, &self.sort_state),
            get_header_with_sort_indicator("Created", NetworkSortField::Created, &self.sort_state),
            get_header_with_sort_indicator("Scope", NetworkSortField::Scope, &self.sort_state),
        ]);

        let widths = constraints![==30%, ==25%, ==15%, ==15%, ==15%];

        let table = Table::new(rows.clone(), widths)
            .header(columns.clone().style(Style::new().bold()))
            .row_highlight_style(Style::new().reversed());

        f.render_stateful_widget(table, table_area, &mut self.list_state);

        if let Some(m) = self.modal.as_mut()
            && let ModalState::Open(_) = m.state
        {
            m.draw(f, area);
        }
    }
}

fn get_network_rows<'a>(networks: &'a [DockerNetwork]) -> Vec<Row<'a>> {
    networks
        .iter()
        .map(|c| {
            Row::new(vec![
                c.id.as_str(),
                c.name.as_str(),
                c.driver.as_str(),
                c.created_at.as_str(),
                c.scope.as_str(),
            ])
        })
        .collect::<Vec<Row<'a>>>()
}

fn get_header_with_sort_indicator(
    header: &str,
    field: NetworkSortField,
    sort_state: &NetworkSortState,
) -> String {
    if sort_state.field == field {
        let indicator = match sort_state.order {
            SortOrder::Ascending => "↑",
            SortOrder::Descending => "↓",
        };
        format!("{} {}", header, indicator)
    } else {
        header.to_string()
    }
}

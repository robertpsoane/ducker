use bollard::Docker;
use color_eyre::eyre::{Context, ContextCompat, Result, bail};
use futures::lock::Mutex as FutureMutex;
use ratatui::{
    Frame,
    layout::Rect,
    // prelude::*,
    style::Style,
    widgets::{Row, Table, TableState},
};
use ratatui_macros::constraints;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::Sender;

use crate::{
    callbacks::delete_volume::DeleteVolume,
    components::{
        boolean_modal::{BooleanModal, ModalState},
        help::{PageHelp, PageHelpBuilder},
        text_input_wrapper::TextInputWrapper,
    },
    config::Config,
    context::AppContext,
    docker::volume::DockerVolume,
    events::{Key, Message, Transition, message::MessageResponse},
    sorting::{SortOrder, SortState, VolumeSortField},
    traits::{Close, Component, ModalComponent, Page},
    ui::{get_field_sort_order, is_field_sorted, render_column_header},
};

const NAME: &str = "Volumes";

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
const ALT_D_KEY: Key = Key::Alt('d');
const SLASH_KEY: Key = Key::Char('/');
const ESC_KEY: Key = Key::Esc;
const ENTER_KEY: Key = Key::Enter;

// Sort keys
const SHIFT_N_KEY: Key = Key::Char('N');
const SHIFT_C_KEY: Key = Key::Char('C');
const SHIFT_M_KEY: Key = Key::Char('M');

type VolumeSortState = SortState<VolumeSortField>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalTypes {
    DeleteVolume,
    ForceDeleteVolume,
}

#[derive(Debug)]
pub struct Volume {
    pub name: String,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
    docker: Docker,
    volumes: Vec<DockerVolume>,
    filtered_volumes: Vec<DockerVolume>,
    list_state: TableState,
    modal: Option<BooleanModal<ModalTypes>>,
    sort_state: VolumeSortState,
    show_dangling: bool,
    table_height: u16,
    is_filtering: bool,
    filter_input: TextInputWrapper,
}

#[async_trait::async_trait]
impl Page for Volume {
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
                    self.filter_volumes(true);
                    self.sort_volumes();
                    return Ok(MessageResponse::Consumed);
                }
                ENTER_KEY => {
                    self.is_filtering = false;
                    return Ok(MessageResponse::Consumed);
                }
                _ => {
                    self.filter_input.update(message)?;
                    self.filter_volumes(true);
                    self.sort_volumes();
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
                self.sort_state.toggle_or_set(VolumeSortField::Driver);
                self.sort_volumes();
                MessageResponse::Consumed
            }
            G_KEY => {
                self.list_state.select(Some(0));
                MessageResponse::Consumed
            }
            SHIFT_G_KEY => {
                self.list_state.select(Some(self.filtered_volumes.len() - 1));
                MessageResponse::Consumed
            }
            SHIFT_N_KEY => {
                self.sort_state.toggle_or_set(VolumeSortField::Name);
                self.sort_volumes();
                MessageResponse::Consumed
            }
            SHIFT_C_KEY => {
                self.sort_state.toggle_or_set(VolumeSortField::Created);
                self.sort_volumes();
                MessageResponse::Consumed
            }
            SHIFT_M_KEY => {
                self.sort_state.toggle_or_set(VolumeSortField::Mountpoint);
                self.sort_volumes();
                MessageResponse::Consumed
            }
            CTRL_D_KEY => match self.delete_volume(false, None, None) {
                Ok(()) => MessageResponse::Consumed,
                Err(_) => MessageResponse::NotConsumed,
            },
            ALT_D_KEY => {
                self.show_dangling = !self.show_dangling;
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
            _ => MessageResponse::NotConsumed,
        };
        Ok(result)
    }

    async fn initialise(&mut self, cx: AppContext) -> Result<()> {
        self.list_state = TableState::default();
        self.list_state.select(Some(0));

        self.refresh().await.context("unable to refresh volumes")?;

        let volume_id: String;
        if let Some(volume) = cx.docker_volume {
            volume_id = volume.name;
        } else if let Some(thing) = cx.describable {
            volume_id = thing.get_id();
        } else {
            return Ok(());
        }

        for (idx, c) in self.filtered_volumes.iter().enumerate() {
            if c.name == volume_id {
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
impl Close for Volume {}

impl Volume {
    #[must_use]
    pub fn new(docker: Docker, tx: Sender<Message<Key, Transition>>, config: Arc<Config>) -> Self {
        let page_help = PageHelpBuilder::new(NAME.to_string(), config.clone())
            .add_input(format!("{CTRL_D_KEY}"), "delete".to_string())
            .add_input(format!("{ALT_D_KEY}"), "dangling".to_string())
            .add_input(format!("{G_KEY}"), "top".to_string())
            .add_input(format!("{SHIFT_G_KEY}"), "bottom".to_string())
            .add_input(format!("{D_KEY}"), "describe".to_string())
            .build();

        Self {
            name: String::from(NAME),
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
            docker,
            volumes: vec![],
            filtered_volumes: vec![],
            list_state: TableState::default(),
            modal: None,
            sort_state: VolumeSortState::default(),
            show_dangling: true,
            table_height: 0,
            is_filtering: false,
            filter_input: TextInputWrapper::new("Filter".to_string(), None),
        }
    }

    async fn refresh(&mut self) -> Result<(), color_eyre::eyre::Error> {
        let mut filters: HashMap<String, Vec<String>> = HashMap::new();
        if self.show_dangling {
            filters.insert("dangling".into(), vec!["true".into()]);
        } else {
            filters.insert("dangling".into(), vec!["false".into()]);
        }

        self.volumes = DockerVolume::list(&self.docker)
            .await
            .context("unable to retrieve list of volumes")?;

        let selected_name = self.get_volume().map(|c| c.name.clone()).ok();
        self.filter_volumes(false);
        // Apply current sort after refresh
        self.sort_volumes();

        if let Some(name) = selected_name {
            if let Some(idx) = self.filtered_volumes.iter().position(|c| c.name == name) {
                self.list_state.select(Some(idx));
            }
        }

        Ok(())
    }

    fn filter_volumes(&mut self, reset_selection: bool) {
        let filter_text = self.filter_input.get_value().to_lowercase();
        self.filtered_volumes = self
            .volumes
            .iter()
            .filter(|c| {
                if filter_text.is_empty() {
                    return true;
                }
                c.name.to_lowercase().contains(&filter_text)
                    || c.driver.to_lowercase().contains(&filter_text)
                    || c.mountpoint.to_lowercase().contains(&filter_text)
            })
            .cloned()
            .collect();
        if reset_selection {
            self.list_state.select(Some(0));
        }
    }

    fn sort_volumes(&mut self) {
        let field = self.sort_state.field;
        let order = self.sort_state.order;

        self.filtered_volumes.sort_by(|a, b| {
            let comparison = match field {
                VolumeSortField::Name => a.name.cmp(&b.name),
                VolumeSortField::Driver => a.driver.cmp(&b.driver),
                VolumeSortField::Mountpoint => a.mountpoint.cmp(&b.mountpoint),
                VolumeSortField::Created => {
                    let a_created = a.created_at.as_deref().unwrap_or("");
                    let b_created = b.created_at.as_deref().unwrap_or("");
                    a_created.cmp(b_created)
                }
            };

            match order {
                SortOrder::Ascending => comparison,
                SortOrder::Descending => comparison.reverse(),
            }
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
                    if let ModalTypes::DeleteVolume = m.discriminator {
                        let msg = "An error occurred deleting this volume; would you like to try to force remove?";
                        self.delete_volume(
                            true,
                            Some(msg.into()),
                            Some(ModalTypes::ForceDeleteVolume),
                        )?;
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
                if !self.filtered_volumes.is_empty() {
                    let len = self.filtered_volumes.len();
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

    fn get_volume(&self) -> Result<&DockerVolume> {
        if let Some(volume_idx) = self.list_state.selected()
            && let Some(volume) = self.filtered_volumes.get(volume_idx)
        {
            return Ok(volume);
        }
        bail!("no container id found");
    }

    fn get_context(&self) -> Result<AppContext> {
        let volume = self.get_volume()?;

        let then = Some(Box::new(Transition::ToVolumePage(AppContext {
            docker_volume: Some(volume.clone()),
            ..Default::default()
        })));

        let cx = AppContext {
            describable: Some(Box::new(volume.clone())),
            then,
            ..Default::default()
        };

        Ok(cx)
    }

    fn delete_volume(
        &mut self,
        force: bool,
        message_override: Option<String>,
        type_override: Option<ModalTypes>,
    ) -> Result<()> {
        if let Ok(volume) = self.get_volume() {
            let name = volume.name.clone();

            let cb = Arc::new(FutureMutex::new(DeleteVolume::new(
                self.docker.clone(),
                volume.clone(),
                force,
            )));

            let mut modal = BooleanModal::<ModalTypes>::new(
                "Delete".into(),
                match type_override {
                    Some(t) => t,
                    None => ModalTypes::DeleteVolume,
                },
            );

            modal.initialise(
                if let Some(m) = message_override {
                    m
                } else {
                    format!("Are you sure you wish to delete volume {name})?")
                },
                Some(cb),
            );
            self.modal = Some(modal);
        } else {
            bail!("Ahhh")
        }
        Ok(())
    }
}

impl Component for Volume {
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
        let rows = get_volume_rows(&self.filtered_volumes);
        let columns = get_header_row(&self.sort_state);

        let widths = constraints![==30%, ==15%, ==30%, ==25%];

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

fn get_volume_rows(volumes: &[DockerVolume]) -> Vec<Row<'static>> {
    volumes
        .iter()
        .map(|c| {
            Row::new(vec![
                c.name.clone(),
                c.driver.clone(),
                c.mountpoint.clone(),
                c.created_at.clone().unwrap_or_default(),
            ])
        })
        .collect::<Vec<Row<'static>>>()
}

fn get_header_row(sort_state: &VolumeSortState) -> Row<'static> {
    let headers = vec![
        render_column_header(
            "Name",
            is_field_sorted(sort_state, &VolumeSortField::Name),
            get_field_sort_order(sort_state, &VolumeSortField::Name)
                .unwrap_or(SortOrder::Ascending),
        ),
        render_column_header(
            "Driver",
            is_field_sorted(sort_state, &VolumeSortField::Driver),
            get_field_sort_order(sort_state, &VolumeSortField::Driver)
                .unwrap_or(SortOrder::Ascending),
        ),
        render_column_header(
            "Mountpoint",
            is_field_sorted(sort_state, &VolumeSortField::Mountpoint),
            get_field_sort_order(sort_state, &VolumeSortField::Mountpoint)
                .unwrap_or(SortOrder::Ascending),
        ),
        render_column_header(
            "Created",
            is_field_sorted(sort_state, &VolumeSortField::Created),
            get_field_sort_order(sort_state, &VolumeSortField::Created)
                .unwrap_or(SortOrder::Ascending),
        ),
    ];

    Row::new(headers)
}

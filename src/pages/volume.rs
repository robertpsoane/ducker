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
    callbacks::delete_volume::DeleteVolume,
    components::{
        boolean_modal::{BooleanModal, ModalState},
        help::{PageHelp, PageHelpBuilder},
    },
    config::Config,
    context::AppContext,
    docker::volume::DockerVolume,
    events::{message::MessageResponse, Key, Message, Transition},
    traits::{Close, Component, ModalComponent, Page},
};

const NAME: &str = "Volumes";

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
    list_state: TableState,
    modal: Option<BooleanModal<ModalTypes>>,
    show_dangling: bool,
}

#[async_trait::async_trait]
impl Page for Volume {
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
                self.list_state.select(Some(self.volumes.len() - 1));
                MessageResponse::Consumed
            }
            CTRL_D_KEY => match self.delete_volume(false, None, None) {
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

        self.refresh().await.context("unable to refresh volumes")?;

        let volume_id: String;
        if let Some(volume) = cx.docker_volume {
            volume_id = volume.name;
        } else if let Some(thing) = cx.describable {
            volume_id = thing.get_id();
        } else {
            return Ok(());
        }

        for (idx, c) in self.volumes.iter().enumerate() {
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
            volumes: vec![],
            list_state: TableState::default(),
            modal: None,
            show_dangling: false,
        }
    }

    async fn refresh(&mut self) -> Result<(), color_eyre::eyre::Error> {
        let mut filters: HashMap<String, Vec<String>> = HashMap::new();
        filters.insert("dangling".into(), vec!["false".into()]);

        self.volumes = DockerVolume::list(&self.docker)
            .await
            .context("unable to retrieve list of volumes")?;
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

    fn increment_list(&mut self) {
        let current_idx = self.list_state.selected();
        match current_idx {
            None => self.list_state.select(Some(0)),
            Some(current_idx) => {
                if !self.volumes.is_empty() && current_idx < self.volumes.len() - 1 {
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

    fn get_volume(&self) -> Result<&DockerVolume> {
        if let Some(volume_idx) = self.list_state.selected() {
            if let Some(volume) = self.volumes.get(volume_idx) {
                return Ok(volume);
            }
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
        let rows = get_volume_rows(&self.volumes);
        let columns = Row::new(vec!["Name", "Driver", "Mountpoint", "Created"]);

        let widths = constraints![==30%, ==15%, ==30%, ==25%];

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

fn get_volume_rows(volumes: &[DockerVolume]) -> Vec<Row> {
    let rows = volumes
        .iter()
        .map(|c| {
            Row::new(vec![
                c.name.clone(),
                c.driver.clone(),
                c.mountpoint.clone(),
                c.created_at.clone().unwrap_or_default(),
            ])
        })
        .collect::<Vec<Row>>();
    rows
}

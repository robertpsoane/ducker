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
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    callbacks::delete_image::DeleteImage,
    components::{
        boolean_modal::{BooleanModal, ModalState},
        help::{PageHelp, PageHelpBuilder},
    },
    config::Config,
    context::AppContext,
    docker::image::DockerImage,
    events::{message::MessageResponse, Key},
    traits::{Close, Component, ModalComponent, Page},
};

const NAME: &str = "Images";

const UP_KEY: Key = Key::Up;
const DOWN_KEY: Key = Key::Down;

const J_KEY: Key = Key::Char('j');
const K_KEY: Key = Key::Char('k');
const CTRL_D_KEY: Key = Key::Ctrl('d');
const D_KEY: Key = Key::Char('d');
const G_KEY: Key = Key::Char('g');
const SHIFT_G_KEY: Key = Key::Char('G');

#[derive(Debug)]
enum ModalTypes {
    DeleteImage,
    ForceDeleteImage,
}

#[derive(Debug)]
pub struct Images {
    pub name: String,
    page_help: Arc<Mutex<PageHelp>>,
    docker: Docker,
    images: Vec<DockerImage>,
    list_state: TableState,
    modal: Option<BooleanModal<ModalTypes>>,
    show_dangling: bool,
}

#[async_trait::async_trait]
impl Page for Images {
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
            D_KEY => {
                self.show_dangling = !self.show_dangling;
                MessageResponse::Consumed
            }
            G_KEY => {
                self.list_state.select(Some(0));
                MessageResponse::Consumed
            }
            SHIFT_G_KEY => {
                self.list_state.select(Some(self.images.len() - 1));
                MessageResponse::Consumed
            }
            CTRL_D_KEY => match self.delete_image(false, None, None) {
                Ok(_) => MessageResponse::Consumed,
                Err(_) => MessageResponse::NotConsumed,
            },

            _ => MessageResponse::NotConsumed,
        };
        Ok(result)
    }

    async fn initialise(&mut self, _: AppContext) -> Result<()> {
        self.list_state = TableState::default();
        self.list_state.select(Some(0));

        self.refresh().await.context("unable to refresh images")?;
        Ok(())
    }

    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

#[async_trait::async_trait]
impl Close for Images {}

impl Images {
    pub fn new(docker: Docker, config: Box<Config>) -> Self {
        let page_help = PageHelpBuilder::new(NAME.into(), config.clone())
            .add_input(format!("{CTRL_D_KEY}"), "delete".into())
            .add_input(format!("{G_KEY}"), "top".into())
            .add_input(format!("{SHIFT_G_KEY}"), "bottom".into())
            .add_input(format!("{D_KEY}"), "dangling".into())
            .build();

        Self {
            name: String::from(NAME),
            page_help: Arc::new(Mutex::new(page_help)),
            docker,
            images: vec![],
            list_state: TableState::default(),
            modal: None,
            show_dangling: false,
        }
    }

    async fn refresh(&mut self) -> Result<(), color_eyre::eyre::Error> {
        let mut filters: HashMap<String, Vec<String>> = HashMap::new();
        filters.insert("dangling".into(), vec!["false".into()]);

        self.images = DockerImage::list(&self.docker, self.show_dangling)
            .await
            .context("unable to retrieve list of images")?;
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
                        self.modal = None
                    }
                }
                Err(e) => {
                    if let ModalTypes::DeleteImage = m.discriminator {
                        let msg = "An error occurred deleting this image; would you like to try to force remove?";
                        self.delete_image(
                            true,
                            Some(msg.into()),
                            Some(ModalTypes::ForceDeleteImage),
                        )?
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
                if !self.images.is_empty() && current_idx < self.images.len() - 1 {
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

    fn get_image(&self) -> Result<&DockerImage> {
        if let Some(image_idx) = self.list_state.selected() {
            if let Some(image) = self.images.get(image_idx) {
                return Ok(image);
            }
        }
        bail!("no container id found");
    }

    fn delete_image(
        &mut self,
        force: bool,
        message_override: Option<String>,
        type_override: Option<ModalTypes>,
    ) -> Result<()> {
        if let Ok(image) = self.get_image() {
            let name = image.name.clone();
            let tag = image.tag.clone();

            let cb = Arc::new(FutureMutex::new(DeleteImage::new(
                self.docker.clone(),
                image.clone(),
                force,
            )));

            let mut modal = BooleanModal::<ModalTypes>::new(
                "Delete".into(),
                match type_override {
                    Some(t) => t,
                    None => ModalTypes::DeleteImage,
                },
            );

            modal.initialise(
                match message_override {
                    Some(m) => m,
                    None => format!("Are you sure you wish to delete container {name}:{tag})?"),
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

impl Component for Images {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let rows = get_image_rows(&self.images);
        let columns = Row::new(vec!["ID", "Name", "Tag", "Created", "Size"]);

        let widths = [
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
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

fn get_image_rows(containers: &[DockerImage]) -> Vec<Row> {
    let rows = containers
        .iter()
        .map(|c| {
            Row::new(vec![
                c.id.clone(),
                c.name.clone(),
                c.tag.clone(),
                c.created.clone(),
                c.size.clone(),
            ])
        })
        .collect::<Vec<Row>>();
    rows
}

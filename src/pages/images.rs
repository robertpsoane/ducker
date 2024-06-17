use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bollard::Docker;
use color_eyre::eyre::{bail, Context, Result};
use ratatui::{
    layout::Rect,
    prelude::*,
    style::Style,
    widgets::{Row, Table, TableState},
    Frame,
};

use crate::{
    components::confirmation_modal::{BooleanOptions, ConfirmationModal, ModalState},
    components::help::PageHelp,
    docker::image::DockerImage,
    events::{message::MessageResponse, Key},
    traits::Component,
    traits::Page,
};

const NAME: &str = "Images";

const UP_KEY: Key = Key::Up;
const DOWN_KEY: Key = Key::Down;

const J_KEY: Key = Key::Char('j');
const K_KEY: Key = Key::Char('k');
const D_KEY: Key = Key::Char('d');
const R_KEY: Key = Key::Char('r');
const S_KEY: Key = Key::Char('s');
const G_KEY: Key = Key::Char('g');
const SHIFT_G_KEY: Key = Key::Char('G');

#[derive(Debug)]
pub struct Images {
    pub name: String,
    pub visible: bool,
    page_help: Arc<Mutex<PageHelp>>,
    docker: Docker,
    images: Vec<DockerImage>,
    list_state: TableState,
    delete_modal: ConfirmationModal<BooleanOptions>,
}

#[async_trait::async_trait]
impl Page for Images {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        if !self.visible {
            return Ok(MessageResponse::NotConsumed);
        }

        self.refresh().await?;

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
                UP_KEY | K_KEY => {
                    self.decrement_list();
                    MessageResponse::Consumed
                }
                DOWN_KEY | J_KEY => {
                    self.increment_list();
                    MessageResponse::Consumed
                }
                D_KEY => {
                    if let Ok(image) = self.get_image() {
                        let name = image.get_full_name();
                        // self.delete_modal
                        //     .initialise(format!("Are you sure you wish to delete image {name}?"));
                        MessageResponse::Consumed
                    } else {
                        MessageResponse::NotConsumed
                    }
                }
                // R_KEY => {
                //     self.start_container()
                //         .await
                //         .context("could not start container")?;
                //     MessageResponse::Consumed
                // }
                // S_KEY => {
                //     self.stop_container()
                //         .await
                //         .context("could not stop container")?;
                //     MessageResponse::Consumed
                // }
                // A_KEY => {
                //     self.attach_container()
                //         .await
                //         .context("could not attach to container")?;
                //     MessageResponse::Consumed
                // }
                // G_KEY => {
                //     self.list_state.select(Some(0));
                //     MessageResponse::Consumed
                // }
                // SHIFT_G_KEY => {
                //     self.list_state.select(Some(self.image_summary.len() - 1));
                //     MessageResponse::Consumed
                // }
                _ => MessageResponse::NotConsumed,
            },
            ModalState::Waiting(_) => {
                let update_res = self.delete_modal.update(message).await?;
                if update_res == MessageResponse::Consumed {
                    if let ModalState::Complete(res) = self.delete_modal.state.clone() {
                        match res {
                            BooleanOptions::Yes => {
                                self.delete_image()
                                    .await
                                    .context("could not delete current image")?;
                                self.delete_modal.reset();
                            }
                            BooleanOptions::No => self.delete_modal.reset(),
                        }
                    }
                }
                MessageResponse::NotConsumed
            }
            ModalState::Complete(_) => {
                self.delete_modal.reset();
                MessageResponse::NotConsumed
            }
        };
        Ok(result)
    }

    async fn initialise(&mut self) -> Result<()> {
        self.list_state = TableState::default();
        self.list_state.select(Some(0));

        self.refresh().await?;
        Ok(())
    }

    async fn set_visible(&mut self) -> Result<()> {
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

impl Images {
    pub async fn new(docker: Docker) -> Result<Self> {
        let page_help = PageHelp::new(NAME.into())
            // .add_input(format!("{}", A_KEY), "attach".into())
            .add_input(format!("{}", D_KEY), "delete".into())
            .add_input(format!("{}", R_KEY), "run".into())
            .add_input(format!("{}", S_KEY), "stop".into())
            .add_input(format!("{}", G_KEY), "to-top".into())
            .add_input(format!("{}", SHIFT_G_KEY), "to-bottom".into());

        Ok(Self {
            name: String::from(NAME),
            page_help: Arc::new(Mutex::new(page_help)),
            visible: false,
            docker,
            images: vec![],
            list_state: TableState::default(),
            delete_modal: ConfirmationModal::<BooleanOptions>::new("Delete".into()),
        })
    }

    async fn refresh(&mut self) -> Result<(), color_eyre::eyre::Error> {
        let mut filters: HashMap<String, Vec<String>> = HashMap::new();
        filters.insert("dangling".into(), vec!["false".into()]);

        self.images = DockerImage::list(&self.docker)
            .await
            .context("unable to retrieve list of images")?;
        Ok(())
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

    async fn delete_image(&mut self) -> Result<Option<()>> {
        self.get_image()?.delete(&self.docker).await?;

        Ok(None)
    }

    // async fn start_container(&mut self) -> Result<Option<()>> {
    //     if let Ok(container) = self.get_container() {
    //         if let Some(container_id) = container.id.clone() {
    //             self.docker
    //                 .start_container::<String>(&container_id, None)
    //                 .await
    //                 .context("failed to start container")?;
    //         }

    //         self.refresh().await?;
    //         return Ok(Some(()));
    //     }
    //     Ok(None)
    // }

    // async fn stop_container(&mut self) -> Result<Option<()>> {
    //     if let Ok(container) = self.get_container() {
    //         if let Some(container_id) = container.id.clone() {
    //             self.docker
    //                 .stop_container(&container_id, None)
    //                 .await
    //                 .context("failed to start container")?;
    //         }

    //         self.refresh().await?;
    //         return Ok(Some(()));
    //     }
    //     Ok(None)
    // }

    // async fn attach_container(&mut self) -> Result<Option<()>> {
    //     if let Ok(container) = self.get_container() {
    //         if let Some(container_id) = container.id.clone() {
    //             self.docker
    //                 .stop_container(&container_id, None)
    //                 .await
    //                 .context("failed to start container")?;
    //         }

    //         self.refresh().await?;
    //         return Ok(Some(()));
    //     }
    //     Ok(None)
    // }
}

impl Component for Images {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let rows = get_image_rows(&self.images);
        let columns = Row::new(vec![
            "ID", "Name", "Tag", //, "Command", "Created", "Status", "Ports", "Names",
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

fn get_image_rows(containers: &[DockerImage]) -> Vec<Row> {
    let rows = containers
        .iter()
        .map(|c| Row::new(vec![c.id.clone(), c.name.clone(), c.tag.clone()]))
        .collect::<Vec<Row>>();
    rows
}

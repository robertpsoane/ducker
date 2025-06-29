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
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::Sender;

use crate::{
    callbacks::delete_image::DeleteImage,
    components::{
        boolean_modal::{BooleanModal, ModalState},
        help::{PageHelp, PageHelpBuilder},
    },
    config::Config,
    context::AppContext,
    docker::image::DockerImage,
    events::{message::MessageResponse, Key, Message, Transition},
    sorting::{ImageSortField, SortOrder, SortState, sort_images_by_name, sort_images_by_tag, sort_images_by_created, sort_images_by_size, sort_images_by_id},
    traits::{Close, Component, ModalComponent, Page},
};

const NAME: &str = "Images";

const UP_KEY: Key = Key::Up;
const DOWN_KEY: Key = Key::Down;

const J_KEY: Key = Key::Char('j');
const K_KEY: Key = Key::Char('k');
const CTRL_D_KEY: Key = Key::Ctrl('d');
const SHIFT_D_KEY: Key = Key::Char('D');
const D_KEY: Key = Key::Char('d');
const G_KEY: Key = Key::Char('g');
const SHIFT_G_KEY: Key = Key::Char('G');

// Sort keys
const SHIFT_N_KEY: Key = Key::Char('N');
const SHIFT_C_KEY: Key = Key::Char('C');
const SHIFT_T_KEY: Key = Key::Char('T');
const SHIFT_S_KEY: Key = Key::Char('S');

type ImageSortState = SortState<ImageSortField>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModalTypes {
    DeleteImage,
    ForceDeleteImage,
}

#[derive(Debug)]
pub struct Images {
    pub name: String,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
    docker: Docker,
    images: Vec<DockerImage>,
    list_state: TableState,
    modal: Option<BooleanModal<ModalTypes>>,
    show_dangling: bool,
    sort_state: ImageSortState,
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
            SHIFT_D_KEY => {
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
            SHIFT_N_KEY => {
                self.sort_state.toggle_or_set(ImageSortField::Name);
                self.sort_images();
                MessageResponse::Consumed
            }
            SHIFT_C_KEY => {
                self.sort_state.toggle_or_set(ImageSortField::Created);
                self.sort_images();
                MessageResponse::Consumed
            }
            SHIFT_T_KEY => {
                self.sort_state.toggle_or_set(ImageSortField::Tag);
                self.sort_images();
                MessageResponse::Consumed
            }
            SHIFT_S_KEY => {
                self.sort_state.toggle_or_set(ImageSortField::Size);
                self.sort_images();
                MessageResponse::Consumed
            }
            CTRL_D_KEY => match self.delete_image(false, None, None) {
                Ok(_) => MessageResponse::Consumed,
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

        self.refresh().await.context("unable to refresh images")?;

        // If a context has been passed in, choose that item in list
        // this ist to allo logs, attach etc to appear to revert to previous
        // state
        // I'm sure there is a more sensible way of doing this...
        let image_id: String;
        if let Some(image) = cx.docker_image {
            image_id = image.id;
        } else if let Some(thing) = cx.describable {
            image_id = thing.get_id();
        } else {
            return Ok(());
        }

        for (idx, c) in self.images.iter().enumerate() {
            if c.id == image_id {
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
impl Close for Images {}

impl Images {
    pub fn new(docker: Docker, tx: Sender<Message<Key, Transition>>, config: Arc<Config>) -> Self {
        let page_help = PageHelpBuilder::new(NAME.to_string(), config.clone())
            .add_input(format!("{CTRL_D_KEY}"), "delete".to_string())
            .add_input(format!("{G_KEY}"), "top".to_string())
            .add_input(format!("{SHIFT_G_KEY}"), "bottom".to_string())
            .add_input(format!("{SHIFT_D_KEY}"), "dangling".to_string())
            .add_input(format!("{D_KEY}"), "describe".to_string())
            .build();

        Self {
            name: String::from(NAME),
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
            docker,
            images: vec![],
            list_state: TableState::default(),
            modal: None,
            show_dangling: false,
            sort_state: ImageSortState::new(ImageSortField::Name),
        }
    }

    async fn refresh(&mut self) -> Result<(), color_eyre::eyre::Error> {
        self.images = DockerImage::list(&self.docker, self.show_dangling)
            .await
            .context("unable to retrieve list of images")?;

        self.sort_images();
        Ok(())
    }

    fn sort_images(&mut self) {
        let field = self.sort_state.field;
        let order = self.sort_state.order;

        self.images.sort_by(|a, b| {
            match field {
                ImageSortField::Id => sort_images_by_id(a, b, order),
                ImageSortField::Name => sort_images_by_name(a, b, order),
                ImageSortField::Tag => sort_images_by_tag(a, b, order),
                ImageSortField::Created => sort_images_by_created(a, b, order),
                ImageSortField::Size => sort_images_by_size(a, b, order),
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

    fn get_context(&self) -> Result<AppContext> {
        let image = self.get_image()?;

        let then = Some(Box::new(Transition::ToImagePage(AppContext {
            docker_image: Some(image.clone()),
            ..Default::default()
        })));

        let cx = AppContext {
            describable: Some(Box::new(image.clone())),
            then,
            ..Default::default()
        };

        Ok(cx)
    }
}

impl Component for Images {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let rows = get_image_rows(&self.images);
        let columns = Row::new(vec![
            get_header_with_sort_indicator("ID", ImageSortField::Id, &self.sort_state),
            get_header_with_sort_indicator("Name", ImageSortField::Name, &self.sort_state),
            get_header_with_sort_indicator("Tag", ImageSortField::Tag, &self.sort_state),
            get_header_with_sort_indicator("Created", ImageSortField::Created, &self.sort_state),
            get_header_with_sort_indicator("Size", ImageSortField::Size, &self.sort_state),
        ]);

        let widths = constraints![==20%, ==20%, ==20%, ==20%, ==20%];

        let table = Table::new(rows.clone(), widths)
            .header(columns.clone().style(Style::new().bold()))
            .row_highlight_style(Style::new().reversed());

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

fn get_header_with_sort_indicator(
    header: &str,
    field: ImageSortField,
    sort_state: &ImageSortState,
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

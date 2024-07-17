use std::sync::Arc;

use bollard::Docker;
use color_eyre::eyre::{Context, Result};
use ratatui::{
    layout::{Layout, Rect},
    style::Style,
    widgets::Block,
    Frame,
};
use ratatui_macros::{horizontal, vertical};
use tokio::sync::mpsc::Sender;

use crate::{
    components::{
        alert_modal::{AlertModal, ModalState},
        command_input::CommandInput,
        footer::Footer,
        header::Header,
        resize_notice::ResizeScreen,
    },
    config::Config,
    events::{key::Key, message::MessageResponse, Message, Transition},
    state::{self, Running},
    traits::{Component, ModalComponent},
    ui::page_manager::PageManager,
};

#[derive(Debug)]
enum ModalType {
    AlertModal,
}

#[derive(Debug)]
pub struct App {
    pub running: state::Running,
    config: Arc<Config>,
    mode: state::Mode,
    blocked: bool,
    resize_screen: ResizeScreen,
    title: Header,
    page_manager: PageManager,
    footer: Footer,
    input_field: CommandInput,
    modal: Option<AlertModal<ModalType>>,
}

impl App {
    pub async fn new(
        tx: Sender<Message<Key, Transition>>,
        docker: Docker,
        config: Config,
    ) -> Result<Self> {
        let config = Arc::new(config);

        let page = state::CurrentPage::default();

        let body = PageManager::new(page.clone(), tx.clone(), docker, config.clone())
            .await
            .context("unable to create new body component")?;

        let app = Self {
            running: state::Running::default(),
            config: config.clone(),
            mode: state::Mode::default(),
            blocked: true,
            resize_screen: ResizeScreen::new(config.clone()),
            title: Header::new(config.clone()),
            page_manager: body,
            footer: Footer::new(config.clone()).await,
            input_field: CommandInput::new(tx, config.prompt.clone()),
            modal: None,
        };
        Ok(app)
    }

    pub async fn update(&mut self, message: Key) -> MessageResponse {
        // Explicitly here and in transition, if there is an error modal, we don't
        // want to allow any event get into the application until the modal is
        // closed.
        // This should be a catch all for any application errors
        // TODO - add more specific modal calls at error-likely points
        if let Some(m) = self.modal.as_mut() {
            if let ModalState::Open(_) = m.state {
                let res = match m.update(message).await {
                    Ok(r) => r,
                    Err(e) => panic!("failed to process failure modal; {e}"),
                };
                if let ModalState::Closed = m.state {
                    self.modal = None;
                }
                return res;
            }
        }

        let res = match self.mode {
            state::Mode::View => self.update_view_mode(message).await,
            state::Mode::TextInput => self.update_text_mode(message).await,
        };

        res.unwrap_or_else(|e| {
            self.handle_error("Error".into(), format!("{e}"));
            MessageResponse::NotConsumed
        })
    }

    pub async fn transition(&mut self, transition: Transition) -> MessageResponse {
        if let Some(m) = self.modal.as_mut() {
            if let ModalState::Open(_) = m.state {
                return MessageResponse::NotConsumed;
            }
        }
        match transition {
            Transition::Quit => {
                self.running = state::Running::Done;
                MessageResponse::Consumed
            }
            Transition::ToViewMode => {
                self.set_mode(state::Mode::View);
                MessageResponse::Consumed
            }
            _ => self
                .page_manager
                .transition(transition)
                .await
                .unwrap_or_else(|e| {
                    self.handle_error("Error".into(), format!("{e}"));
                    MessageResponse::NotConsumed
                }),
        }
    }

    async fn update_view_mode(&mut self, message: Key) -> Result<MessageResponse> {
        if let MessageResponse::Consumed = self.page_manager.update(message).await? {
            return Ok(MessageResponse::Consumed);
        }

        match message {
            Key::Char(':') => {
                self.set_mode(state::Mode::TextInput);
                Ok(MessageResponse::Consumed)
            }
            Key::Char('Q') | Key::Char('q') => {
                self.running = Running::Done;
                Ok(MessageResponse::Consumed)
            }
            _ => Ok(MessageResponse::NotConsumed),
        }
    }

    async fn update_text_mode(&mut self, message: Key) -> Result<MessageResponse> {
        let result = match message {
            Key::Esc => {
                self.set_mode(state::Mode::View);
                MessageResponse::Consumed
            }
            _ => self.input_field.update(message).await.unwrap(),
        };
        Ok(result)
    }

    fn set_mode(&mut self, mode: state::Mode) {
        self.mode = mode.clone();
        match mode {
            state::Mode::TextInput => self.input_field.initialise(),
            state::Mode::View => {}
        }
    }

    fn handle_error(&mut self, title: String, msg: String) {
        let mut modal = AlertModal::new(title, ModalType::AlertModal);
        modal.initialise(msg);
        self.modal = Some(modal)
    }

    pub fn draw(&mut self, f: &mut Frame<'_>) {
        // Short circuits drawing the app if the frame is too small;
        let area: Rect = f.size();

        f.render_widget(
            Block::new().style(Style::new().bg(self.config.theme.background())),
            area,
        );

        if area.height < self.resize_screen.min_height || area.width < self.resize_screen.min_width
        {
            self.blocked = true;
            self.resize_screen.draw(f, area);
            return;
        } else {
            self.blocked = false
        }

        let layout: Layout;
        let top: Rect;

        let page: Rect;
        let footer: Rect;
        match self.mode {
            state::Mode::TextInput => {
                let text_input: Rect;
                layout = vertical![==5, ==3, >=0, ==1];
                [top, text_input, page, footer] = layout.areas(f.size());
                self.input_field.draw(f, text_input);
            }
            _ => {
                layout = vertical![==5, >=0, ==1];
                [top, page, footer] = layout.areas(f.size());
            }
        }

        let [title, right_space] = horizontal![==50, >=0].areas(top);

        self.title.draw(f, title);
        self.page_manager.draw(f, page);
        self.page_manager.draw_help(f, right_space);
        self.footer.draw(f, footer);

        if let Some(m) = self.modal.as_mut() {
            if let ModalState::Open(_) = m.state {
                m.draw(f, area)
            }
        }
    }
}

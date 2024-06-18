use color_eyre::eyre::{Context, Ok, Result};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    Frame,
};
use tokio::sync::mpsc::Sender;

use crate::{
    components::{footer::Footer, header::Header, input_field::InputField},
    events::{key::Key, message::MessageResponse, Message, Transition},
    state,
    traits::Component,
    ui::page_manager::PageManager,
};

#[derive(Debug)]
pub struct App {
    pub running: state::Running,
    mode: state::Mode,
    title: Header,
    page_manager: PageManager,
    footer: Footer,
    input_field: InputField,
}

impl App {
    pub async fn new(tx: Sender<Message<Key, Transition>>) -> Result<Self> {
        let page = state::CurrentPage::default();

        let body = PageManager::new(page.clone(), tx.clone())
            .await
            .context("unable to create new body component")?;

        let app = Self {
            running: state::Running::default(),
            mode: state::Mode::default(),
            title: Header::default(),
            page_manager: body,
            footer: Footer::default(),
            input_field: InputField::new(tx),
        };
        Ok(app)
    }

    pub async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        match self.mode {
            state::Mode::View => self.update_view_mode(message).await,
            state::Mode::TextInput => self.update_text_mode(message).await,
        }
    }

    pub async fn transition(&mut self, transition: Transition) -> Result<MessageResponse> {
        let result = match transition {
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
                .context("page manager failed to transition")?,
        };
        Ok(result)
    }

    async fn update_view_mode(&mut self, message: Key) -> Result<MessageResponse> {
        if let MessageResponse::Consumed = self
            .page_manager
            .update(message)
            .await
            .context("unable to update body")?
        {
            return Ok(MessageResponse::Consumed);
        }

        match message {
            Key::Char('q') | Key::Char('Q') => {
                self.running = state::Running::Done;
                Ok(MessageResponse::Consumed)
            }
            Key::Char(':') => {
                self.set_mode(state::Mode::TextInput);
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

    pub fn draw(&mut self, f: &mut Frame<'_>) {
        let layout: Layout;
        let top: Rect;

        let page: Rect;
        let footer: Rect;
        match self.mode {
            state::Mode::TextInput => {
                let text_input: Rect;
                layout = Layout::vertical([
                    Constraint::Length(5),
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ]);
                [top, text_input, page, footer] = layout.areas(f.size());
                self.input_field.draw(f, text_input);
            }
            _ => {
                layout = Layout::vertical([
                    Constraint::Length(5),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ]);
                [top, page, footer] = layout.areas(f.size());
            }
        }

        let [_left_space, title, right_space] = Layout::horizontal(vec![
            Constraint::Min(0),
            Constraint::Length(50),
            Constraint::Min(0),
        ])
        .areas(top);

        self.title.draw(f, title);
        self.page_manager.draw(f, page);
        self.page_manager.draw_help(f, right_space);
        self.footer.draw(f, footer)
    }
}

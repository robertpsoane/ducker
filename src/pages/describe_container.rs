use std::sync::{Arc, Mutex};

use bollard::Docker;

use color_eyre::eyre::{bail, Result};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::Sender;

use crate::config::Config;
use crate::context::AppContext;
use crate::docker::container::DockerContainer;
use crate::traits::Close;
use crate::{
    components::help::{PageHelp, PageHelpBuilder},
    events::{message::MessageResponse, Key, Message, Transition},
    traits::{Component, Page},
};

const NAME: &str = "Describe";

const UP_KEY: Key = Key::Up;
const DOWN_KEY: Key = Key::Down;

const J_KEY: Key = Key::Char('j');
const K_KEY: Key = Key::Char('k');

#[derive(Debug)]
pub struct DescribeContainer {
    _docker: Docker,
    config: Box<Config>,
    container: Option<DockerContainer>,
    container_summary: Option<Vec<String>>,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
    scroll: u16,
}

impl DescribeContainer {
    pub fn new(docker: Docker, tx: Sender<Message<Key, Transition>>, config: Box<Config>) -> Self {
        let page_help = PageHelpBuilder::new(NAME.into(), config.clone()).build();

        Self {
            _docker: docker,
            config,
            container: None,
            container_summary: None,
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
            scroll: 0,
        }
    }

    fn down(&mut self) {
        self.scroll += 1;
    }

    fn up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    fn resolve_scroll(&mut self, height: &u16, n_lines: &u16) -> u16 {
        let max_scroll = n_lines - (height / 2);
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        };
        self.scroll
    }

    fn set_summary(&mut self, summary: String) {
        self.container_summary = Some(summary.lines().map(String::from).collect());
    }
}

#[async_trait::async_trait]
impl Page for DescribeContainer {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        let res = match message {
            UP_KEY | K_KEY => {
                self.up();
                MessageResponse::Consumed
            }
            DOWN_KEY | J_KEY => {
                self.down();
                MessageResponse::Consumed
            }
            Key::Esc => {
                self.tx
                    .send(Message::Transition(Transition::ToContainerPage(
                        AppContext {
                            docker_container: self.container.clone(),
                            ..Default::default()
                        },
                    )))
                    .await?;
                MessageResponse::Consumed
            }
            _ => MessageResponse::NotConsumed,
        };

        Ok(res)
    }

    async fn initialise(&mut self, cx: AppContext) -> Result<()> {
        let container = match cx.docker_container.clone() {
            Some(c) => c,
            None => {
                bail!("no docker container")
            }
        };
        self.container = Some(container);

        let summary = match serde_yml::to_string(&self.container) {
            Ok(s) => s,
            Err(_) => {
                bail!("failed to parse container summary")
            }
        };
        self.set_summary(summary);
        Ok(())
    }

    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

#[async_trait::async_trait]
impl Close for DescribeContainer {}

impl Component for DescribeContainer {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        if self.container_summary.is_none() {
            return;
        }
        let container_summary = self.container_summary.as_ref().unwrap();
        let lines: Vec<Line> = container_summary
            .iter()
            .map(|l| {
                let l = l.clone();

                let mut row = l.splitn(2, ':');
                let key = String::from(row.next().unwrap_or(""));
                let val = String::from(row.next().unwrap_or(""));

                let key_style = Style::default().fg(self.config.theme.footer());

                Line::from(vec![
                    Span::from(key.clone()).style(key_style),
                    Span::from(":"),
                    Span::from(val.clone()),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(lines);

        let n_lines = paragraph.line_count(area.width) as u16;

        let scroll = self.resolve_scroll(&area.height, &n_lines);

        let paragraph = paragraph.scroll((scroll, 0));
        f.render_widget(paragraph, area)
    }
}

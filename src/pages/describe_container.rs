use std::sync::{Arc, Mutex};

use bollard::Docker;

use color_eyre::eyre::{bail, Result};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::{layout::Rect, Frame};
use tokio::sync::mpsc::Sender;

use crate::config::Config;
use crate::context::AppContext;
use crate::docker::container_summary::DockerContainerSummary;
use crate::docker::container_verbose::DockerContainerVerbose;
use crate::traits::Close;
use crate::{
    components::help::{PageHelp, PageHelpBuilder},
    events::{message::MessageResponse, Key, Message, Transition},
    traits::{Component, Page},
};

const NAME: &str = "Describe";

const ESC_KEY: Key = Key::Esc;

#[derive(Debug)]
pub struct DescribeContainer {
    docker: Docker,
    config: Box<Config>,
    container: Option<DockerContainerVerbose>,
    container_summary: Option<String>,
    tx: Sender<Message<Key, Transition>>,
    page_help: Arc<Mutex<PageHelp>>,
}

impl DescribeContainer {
    pub fn new(docker: Docker, tx: Sender<Message<Key, Transition>>, config: Box<Config>) -> Self {
        let page_help = PageHelpBuilder::new(NAME.into(), config.clone()).build();

        Self {
            docker,
            config,
            container: None,
            container_summary: None,
            tx,
            page_help: Arc::new(Mutex::new(page_help)),
        }
    }
}

#[async_trait::async_trait]
impl Page for DescribeContainer {
    async fn update(&mut self, message: Key) -> Result<MessageResponse> {
        let res = match message {
            Key::Esc => {
                self.tx
                    .send(Message::Transition(Transition::ToContainerPage(
                        AppContext {
                            docker_container: self.container.to_summary().clone(),
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
        let c = container.verbose(&self.docker).await?;
        self.container = Some(c);

        let summary = match serde_yml::to_string(&self.container) {
            Ok(s) => s,
            Err(_) => {
                bail!("failed to parse container summary")
            }
        };
        self.container_summary = Some(summary);
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
        let paragraph = Paragraph::new(Text::from(self.container_summary.clone().unwrap()));
        f.render_widget(paragraph, area)
    }
}

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use bollard::Docker;

use color_eyre::eyre::{Result, bail};
use itertools::Itertools;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Scrollbar, ScrollbarOrientation};
use ratatui::{Frame, layout::Rect};
use tokio::sync::mpsc::Sender;
use tui_tree_widget::{Tree, TreeItem, TreeState};
use uuid::Uuid;

use crate::config::Config;
use crate::context::AppContext;
use crate::docker::traits::{Describe, DescribeSection};
use crate::traits::Close;
use crate::{
    components::help::{PageHelp, PageHelpBuilder},
    events::{Key, Message, Transition, message::MessageResponse},
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
    config: Arc<Config>,
    thing: Option<Box<dyn Describe>>,
    thing_summary: Option<Vec<DescribeSection>>,
    tx: Sender<Message<Key, Transition>>,
    cx: Option<AppContext>,
    page_help: Arc<Mutex<PageHelp>>,
    scroll: u16,
    tree_state: TreeState<Uuid>,
}

impl DescribeContainer {
    pub fn new(docker: Docker, tx: Sender<Message<Key, Transition>>, config: Arc<Config>) -> Self {
        let page_help = Self::build_page_help(config.clone(), None);

        Self {
            _docker: docker,
            config,
            thing: None,
            thing_summary: None,
            tx,
            cx: None,
            page_help: Arc::new(Mutex::new(page_help)),
            scroll: 0,
            tree_state: TreeState::default(),
        }
    }

    fn build_page_help(config: Arc<Config>, name: Option<String>) -> PageHelp {
        let page_name = if let Some(name) = name {
            name
        } else {
            NAME.into()
        };
        PageHelpBuilder::new(page_name, config).build()
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
        let max_scroll = if *n_lines < (height / 2) {
            0
        } else {
            n_lines - (height / 2)
        };
        if self.scroll > max_scroll {
            self.scroll = max_scroll;
        };
        self.scroll
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
                let transition = match self.cx.clone() {
                    Some(cx) => match cx.then {
                        Some(tr) => *tr.clone(),
                        None => Transition::ToContainerPage(AppContext {
                            describable: self.thing.clone(),
                            ..Default::default()
                        }),
                    },
                    None => Transition::ToContainerPage(AppContext {
                        describable: self.thing.clone(),
                        ..Default::default()
                    }),
                };

                self.tx.send(Message::Transition(transition)).await?;
                MessageResponse::Consumed
            }
            _ => MessageResponse::NotConsumed,
        };

        Ok(res)
    }

    async fn initialise(&mut self, cx: AppContext) -> Result<()> {
        let thing = match cx.describable.clone() {
            Some(c) => c,
            None => {
                bail!("no docker container")
            }
        };
        self.thing_summary = Some(thing.describe()?);
        let page_name = format!("{NAME} ({})", thing.get_name());
        self.page_help = Arc::new(Mutex::new(Self::build_page_help(
            self.config.clone(),
            Some(page_name),
        )));
        self.thing = Some(thing);
        self.cx = Some(cx);

        Ok(())
    }

    fn get_help(&self) -> Arc<Mutex<PageHelp>> {
        self.page_help.clone()
    }
}

#[async_trait::async_trait]
impl Close for DescribeContainer {}

fn section_to_tree_item<'a>(
    state: &mut TreeState<Uuid>,
    section: &'a DescribeSection,
    section_style: &Style,
    key_style: &Style,
) -> TreeItem<'a, Uuid> {
    let items: Vec<TreeItem<Uuid>> = section
        .items
        .iter()
        .map(|item| {
            let line = Line::from(vec![
                Span::from(&item.name).style(*key_style),
                Span::from(": "),
                Span::from(&item.value),
            ]);
            TreeItem::new_leaf(item.id, line)
        })
        .collect();

    let item = TreeItem::new(
        section.id,
        Span::from(&section.name).style(*section_style),
        items,
    )
    .expect("all items should be unique");
    state.open(vec![section.id]);
    item
}

impl Component for DescribeContainer {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        if self.thing_summary.is_none() {
            return;
        }
        if let Some(summary) = &self.thing_summary {
            let section_style = Style::default().fg(self.config.theme.footer());
            let key_style = Style::default().fg(self.config.theme.footer());
            let tree = summary
                .iter()
                .map(|section| {
                    section_to_tree_item(&mut self.tree_state, section, &section_style, &key_style)
                })
                .collect_vec();

            let widget = Tree::new(tree.as_slice())
                .expect("all item identifiers are unique")
                .experimental_scrollbar(Some(
                    Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(None)
                        .track_symbol(None)
                        .end_symbol(None),
                ))
                .highlight_style(
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(widget, area, &mut self.tree_state);
        }
    }
}

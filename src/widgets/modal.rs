use ratatui::{
    layout::{Constraint, Layout},
    style::Style,
    text::{Line, Span, Text},
    widgets::{block::Title, Block, Clear, Paragraph, Widget, Wrap},
};

const UPPER_PAD_SIZE: u16 = 1;
const MID_PAD_SIZE: u16 = 1;

pub struct ModalWidget<'a> {
    title: Title<'a>,
    prompt: Paragraph<'a>,
    opts: Vec<Span<'a>>,
    width: u16,
    height: u16,
}

impl<'a> ModalWidget<'a> {
    pub fn new(title: Title<'a>, prompt: Paragraph<'a>, opts: Vec<Span<'a>>) -> Self {
        Self {
            title,
            prompt,
            opts,
            ..Default::default()
        }
    }
}

impl<'a> Default for ModalWidget<'a> {
    fn default() -> Self {
        Self {
            title: Title::from(""),
            prompt: Paragraph::new(Text::from("")),
            opts: vec![],
            width: 60,
            height: 10,
        }
    }
}

impl<'a> Widget for ModalWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let width = self.width;
        let height = self.height;

        let [_, area, _] = Layout::horizontal(vec![
            Constraint::Min(0),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .areas(area);

        let [_, area, _] = Layout::vertical(vec![
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .areas(area);

        let block = Block::bordered()
            .title(self.title)
            .border_style(Style::default())
            .style(Style::default());

        let inner_block = block.inner(area);

        let vertical_layout = Layout::vertical(vec![
            Constraint::Length(UPPER_PAD_SIZE),
            Constraint::Min(0),
            Constraint::Length(MID_PAD_SIZE),
            Constraint::Min(0),
        ]);

        let [_, top, _, bottom] = vertical_layout.areas(inner_block);

        Clear.render(inner_block, buf);
        block.render(area, buf);

        self.prompt.wrap(Wrap { trim: true }).render(top, buf);

        Line::from(self.opts).centered().render(bottom, buf);
    }
}

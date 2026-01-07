use std::fmt::Debug;

use color_eyre::eyre::Result;
use ratatui::{Frame, layout::Rect};

use crate::events::{Key, message::MessageResponse};

pub trait Component: Debug {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect);
}

#[async_trait::async_trait]
pub trait ModalComponent: Component + Send {
    async fn update(&mut self, message: Key) -> Result<MessageResponse>;
}

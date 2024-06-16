use std::{
    any::Any,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use color_eyre::eyre::Result;

use crate::{
    components::help::PageHelp,
    events::{message::MessageResponse, Key},
    traits::Component,
};

#[async_trait]
pub trait Page: Component + Debug + Any {
    async fn update(&mut self, message: Key) -> Result<MessageResponse>;
    async fn initialise(&mut self) -> Result<()>;
    async fn set_visible(&mut self) -> Result<()>;
    async fn set_invisible(&mut self) -> Result<()>;
    fn get_help(&self) -> Arc<Mutex<PageHelp>>;
}

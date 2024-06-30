use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use color_eyre::eyre::Result;

use crate::{
    components::help::PageHelp,
    context::AppContext,
    events::{message::MessageResponse, Key},
    traits::Component,
};

#[async_trait]
pub trait Page: Component + Close + Debug + Send + Sync {
    async fn update(&mut self, message: Key) -> Result<MessageResponse>;
    async fn initialise(&mut self, cx: AppContext) -> Result<()>;
    fn get_help(&self) -> Arc<Mutex<PageHelp>>;
}

#[async_trait]
pub trait Close: Debug + Send + Sync {
    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

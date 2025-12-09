use crate::{
    events::{Key, Message, Transition},
    traits::Callback,
};
use async_trait::async_trait;
use bollard::query_parameters::PruneNetworksOptionsBuilder;
use color_eyre::eyre::Result;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct PruneNetworks {
    docker: bollard::Docker,

    tx: Sender<Message<Key, Transition>>,
}

impl PruneNetworks {
    pub fn new(docker: bollard::Docker, tx: Sender<Message<Key, Transition>>) -> Self {
        Self { docker, tx }
    }
}

#[async_trait]
impl Callback for PruneNetworks {
    async fn call(&self) -> Result<()> {
        if let Err(err) = self
            .docker
            .prune_networks(Some(PruneNetworksOptionsBuilder::new().build()))
            .await
        {
            let msg = format!("Failed to prune networks: {err}");
            let _ = self.tx.send(Message::Error(msg)).await;
        }
        Ok(())
    }
}

use crate::{
    docker::container::DockerContainer,
    events::{Key, Message, Transition},
    traits::Callback,
};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct DeleteContainer {
    docker: bollard::Docker,
    container: DockerContainer,
    force: bool,
    tx: Sender<Message<Key, Transition>>,
}

impl DeleteContainer {
    pub fn new(
        docker: bollard::Docker,
        container: DockerContainer,
        force: bool,
        tx: Sender<Message<Key, Transition>>,
    ) -> Self {
        Self {
            docker,
            container,
            force,
            tx,
        }
    }
}
#[async_trait]
impl Callback for DeleteContainer {
    async fn call(&self) -> Result<()> {
        let container = self.container.clone();
        let docker = self.docker.clone();
        let force = self.force;
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let message = if container.delete(&docker, force).await.is_ok() {
                Message::Tick
            } else {
                let msg = format!("Failed to delete container {}", container.id);
                Message::Error(msg)
            };
            let _ = tx.send(message).await;
        });
        Ok(())
    }
}

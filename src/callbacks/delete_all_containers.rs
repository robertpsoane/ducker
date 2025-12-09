use crate::{
    docker::container::DockerContainer,
    events::{Key, Message, Transition},
    traits::Callback,
};
use async_trait::async_trait;
use color_eyre::eyre::Result;
use futures::future::join_all;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct DeleteAllContainers {
    docker: bollard::Docker,
    force: bool,
    tx: Sender<Message<Key, Transition>>,
}

impl DeleteAllContainers {
    pub fn new(docker: bollard::Docker, force: bool, tx: Sender<Message<Key, Transition>>) -> Self {
        Self { docker, force, tx }
    }
}

#[async_trait]
impl Callback for DeleteAllContainers {
    async fn call(&self) -> Result<()> {
        let docker = self.docker.clone();
        let force = self.force;
        let tx = self.tx.clone();
        tokio::spawn(async move {
            match DockerContainer::list(&docker).await {
                Err(err) => {
                    let msg = format!("Failed to get list of containers:{err}");
                    let _ = tx.send(Message::Error(msg)).await;
                }
                Ok(containers) => {
                    let handlers = containers.into_iter().map(|container| {
                        let docker = docker.clone();
                        let tx = tx.clone();
                        async move {
                            let message = if container.delete(&docker, force).await.is_ok() {
                                Message::Tick
                            } else {
                                let msg = format!("Failed to delete container {}", container.id);
                                Message::Error(msg)
                            };
                            let _ = tx.send(message).await;
                        }
                    });
                    join_all(handlers).await;
                }
            };
        });
        Ok(())
    }
}

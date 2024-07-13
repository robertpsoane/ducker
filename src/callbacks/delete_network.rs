use crate::{docker::network::DockerNetwork, traits::Callback};
use async_trait::async_trait;
use color_eyre::eyre::Result;

#[derive(Debug)]
pub struct DeleteNetwork {
    docker: bollard::Docker,
    network: DockerNetwork,
}

impl DeleteNetwork {
    pub fn new(docker: bollard::Docker, network: DockerNetwork) -> Self {
        Self { docker, network }
    }
}

#[async_trait]
impl Callback for DeleteNetwork {
    async fn call(&self) -> Result<()> {
        let _ = self.network.delete(&self.docker).await?;
        Ok(())
    }
}

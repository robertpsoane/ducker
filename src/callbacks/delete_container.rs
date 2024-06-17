use crate::{docker::container::DockerContainer, traits::Callback};
use async_trait::async_trait;
use color_eyre::eyre::Result;

#[derive(Debug)]
pub struct DeleteContainer {
    docker: bollard::Docker,
    container: DockerContainer,
    force: bool,
}

impl DeleteContainer {
    pub fn new(docker: bollard::Docker, container: DockerContainer, force: bool) -> Self {
        Self {
            docker,
            container,
            force,
        }
    }
}
#[async_trait]
impl Callback for DeleteContainer {
    async fn call(&self) -> Result<()> {
        let _ = self.container.delete(&self.docker, self.force).await?;
        Ok(())
    }
}

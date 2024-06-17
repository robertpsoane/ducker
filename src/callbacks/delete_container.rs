use crate::{docker::container::DockerContainer, traits::Callback};
use async_trait::async_trait;

#[derive(Debug)]
pub struct DeleteContainer {
    docker: bollard::Docker,
    container: DockerContainer,
}

impl DeleteContainer {
    pub fn new(docker: bollard::Docker, container: DockerContainer) -> Self {
        Self { docker, container }
    }
}
#[async_trait]
impl Callback for DeleteContainer {
    async fn call(&self) {
        let _ = self.container.delete(&self.docker).await;
    }
}

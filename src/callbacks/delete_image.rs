use crate::{docker::image::DockerImage, traits::Callback};
use async_trait::async_trait;

#[derive(Debug)]
pub struct DeleteImage {
    docker: bollard::Docker,
    image: DockerImage,
}

impl DeleteImage {
    pub fn new(docker: bollard::Docker, image: DockerImage) -> Self {
        Self { docker, image }
    }
}
#[async_trait]
impl Callback for DeleteImage {
    async fn call(&self) {
        let _ = self.image.delete(&self.docker).await;
    }
}

use crate::{docker::image::DockerImage, traits::Callback};
use async_trait::async_trait;
use color_eyre::eyre::Result;

#[derive(Debug)]
pub struct DeleteImage {
    docker: bollard::Docker,
    image: DockerImage,
    force: bool,
}

impl DeleteImage {
    pub fn new(docker: bollard::Docker, image: DockerImage, force: bool) -> Self {
        Self {
            docker,
            image,
            force,
        }
    }
}

#[async_trait]
impl Callback for DeleteImage {
    async fn call(&self) -> Result<()> {
        let _ = self.image.delete(&self.docker, self.force).await?;
        Ok(())
    }
}

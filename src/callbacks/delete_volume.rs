use crate::{docker::volume::DockerVolume, traits::Callback};
use async_trait::async_trait;
use color_eyre::eyre::Result;

#[derive(Debug)]
pub struct DeleteVolume {
    docker: bollard::Docker,
    volume: DockerVolume,
    force: bool,
}

impl DeleteVolume {
    pub fn new(docker: bollard::Docker, volume: DockerVolume, force: bool) -> Self {
        Self {
            docker,
            volume,
            force,
        }
    }
}

#[async_trait]
impl Callback for DeleteVolume {
    async fn call(&self) -> Result<()> {
        let _ = self.volume.delete(&self.docker, self.force).await?;
        Ok(())
    }
}

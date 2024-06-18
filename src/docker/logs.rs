use bollard::container::LogsOptions;
use color_eyre::eyre::Result;

use super::container::DockerContainer;

#[derive(Debug, Clone)]
pub struct DockerLogs {
    container: DockerContainer,
}

impl DockerLogs {
    pub fn new(container: DockerContainer) -> Self {
        DockerLogs { container }
    }

    pub fn from(container: DockerContainer) -> Self {
        Self::new(container)
    }

    pub async fn logs(&self, docker: &bollard::Docker) -> Result<()> {
        let logstream = docker.logs(
            &self.container.id,
            Some(LogsOptions::<String> {
                follow: true,
                ..Default::default()
            }),
        );
        Ok(())
    }
}

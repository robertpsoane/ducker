use bollard::container::LogsOptions;
use futures::{Stream, StreamExt};

use super::container_summary::DockerContainerSummary;

#[derive(Debug, Clone)]
pub struct DockerLogs {
    container: DockerContainerSummary,
}

impl DockerLogs {
    pub fn new(container: DockerContainerSummary) -> Self {
        DockerLogs { container }
    }

    pub fn from(container: DockerContainerSummary) -> Self {
        Self::new(container)
    }

    pub fn get_log_stream(&self, docker: &bollard::Docker, tail: u8) -> impl Stream<Item = String> {
        let logstream = docker
            .logs(
                &self.container.id,
                Some(LogsOptions::<String> {
                    follow: true,
                    stdout: true,
                    stderr: true,
                    tail: tail.to_string(),
                    ..Default::default()
                }),
            )
            .filter_map(|res| async move {
                Some(match res {
                    Ok(r) => format!("{r}"),
                    Err(err) => format!("{err}"),
                })
            });

        Box::pin(logstream)
    }
}

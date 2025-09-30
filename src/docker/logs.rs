use bollard::query_parameters::LogsOptionsBuilder;
use futures::{Stream, StreamExt};

use super::container::DockerContainer;

#[derive(Debug, Clone)]
pub struct StreamOptions {
    pub tail: String,
    pub all: bool,
}

impl Default for StreamOptions {
    fn default() -> Self {
        Self {
            tail: "50".into(),
            all: false,
        }
    }
}

impl From<StreamOptions> for bollard::query_parameters::LogsOptions {
    fn from(val: StreamOptions) -> Self {
        let mut builder = LogsOptionsBuilder::default();
        builder = builder
            .follow(true)
            .stdout(true)
            .stderr(true)
            .tail(&val.tail);
        if val.all {
            builder = builder.tail("all");
        }
        builder.build()
    }
}

#[derive(Debug, Clone)]
pub struct DockerLogs {
    pub container: DockerContainer,
}

impl DockerLogs {
    pub fn new(container: DockerContainer) -> Self {
        DockerLogs { container }
    }

    pub fn from(container: DockerContainer) -> Self {
        Self::new(container)
    }

    pub fn get_log_stream(
        &self,
        docker: &bollard::Docker,
        stream_options: StreamOptions,
    ) -> impl Stream<Item = String> {
        let opts: bollard::query_parameters::LogsOptions = stream_options.into();
        let logstream = docker
            .logs(&self.container.id, Some(opts))
            .filter_map(|res| async move {
                Some(match res {
                    Ok(r) => format!("{r}"),
                    Err(err) => format!("{err}"),
                })
            });

        Box::pin(logstream)
    }
}

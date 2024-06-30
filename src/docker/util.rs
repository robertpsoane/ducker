use bollard::{Docker, API_DEFAULT_VERSION};
use color_eyre::eyre::{Context, Result};

use super::container::DockerContainer;

pub async fn new_local_docker_connection(socket_path: &str) -> Result<Docker> {
    let docker = bollard::Docker::connect_with_socket(socket_path, 120, API_DEFAULT_VERSION)
        .with_context(|| "unable to connect to local docker socket")?;

    DockerContainer::list(&docker)
        .await
        .context("unable to connect to local docker socket")?;
    Ok(docker)
}

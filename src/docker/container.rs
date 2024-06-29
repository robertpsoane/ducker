use bollard::container::{ListContainersOptions, RemoveContainerOptions};
use chrono::prelude::DateTime;
use chrono::Local;
use color_eyre::eyre::{Context, Result};
use std::time::{Duration, UNIX_EPOCH};
use tokio::process::Command;

use bollard::secret::ContainerSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockerContainer {
    pub id: String,
    pub image: String,
    pub command: String,
    pub created: String,
    pub status: String,
    pub ports: String,
    pub names: String,
    pub running: bool,
}

impl DockerContainer {
    pub fn from(c: ContainerSummary) -> Self {
        let ports = match c.ports.clone() {
            Some(p) => p
                .into_iter()
                .map(|p| {
                    let ip = p.ip.unwrap_or_default();
                    let private_port = p.private_port.to_string();
                    let public_port = match p.public_port {
                        Some(port) => port.to_string(),
                        None => String::new(),
                    };
                    let typ = match p.typ {
                        Some(t) => format!("{:?}", t),
                        None => String::new(),
                    };

                    format!("{}:{}:{}:{}", ip, private_port, public_port, typ)
                })
                .collect::<Vec<String>>()
                .join(", "),
            None => "".into(),
        };
        let datetime = DateTime::<Local>::from(
            UNIX_EPOCH
                + Duration::from_secs(c.created.unwrap_or_default().try_into().unwrap_or_default()),
        )
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

        let running = matches!(c.state.unwrap_or_default().as_str(), "running");

        let names = c
            .names
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|n| n.strip_prefix('/').unwrap_or_default().into())
            .collect::<Vec<String>>()
            .join(", ");

        Self {
            id: c.id.clone().unwrap_or_default(),
            image: c.image.clone().unwrap_or_default(),
            command: c.command.clone().unwrap_or_default(),
            created: datetime,
            status: c.status.clone().unwrap_or_default(),
            ports,
            names,
            running,
        }
    }

    pub async fn list(docker: &bollard::Docker) -> Result<Vec<Self>> {
        let containers = docker
            .list_containers(Some(ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await
            .context("unable to retrieve list of containers")?
            .into_iter()
            .map(Self::from)
            .collect();

        Ok(containers)
    }

    pub async fn delete(&self, docker: &bollard::Docker, force: bool) -> Result<()> {
        let opt = RemoveContainerOptions {
            force,
            ..Default::default()
        };
        docker.remove_container(&self.id, Some(opt)).await?;
        Ok(())
    }

    pub async fn start(&self, docker: &bollard::Docker) -> Result<()> {
        docker
            .start_container::<String>(&self.id, None)
            .await
            .context("failed to start container")?;

        Ok(())
    }

    pub async fn stop(&self, docker: &bollard::Docker) -> Result<()> {
        docker
            .stop_container(&self.id, None)
            .await
            .context("failed to stop container")?;
        Ok(())
    }

    pub async fn attach(&self, cmd: &str) -> Result<()> {
        Command::new("clear").spawn()?.wait().await?;

        Command::new("docker")
            .arg("exec")
            .arg("-it")
            .arg(&self.names)
            .arg(cmd)
            .spawn()?
            .wait()
            .await?;

        Command::new("clear").spawn()?.wait().await?;
        Ok(())
    }
}

use bollard::container::{InspectContainerOptions, ListContainersOptions, RemoveContainerOptions};
use byte_unit::{Byte, UnitType};
use chrono::prelude::DateTime;
use chrono::Local;
use color_eyre::eyre::{Context, Result};
use serde::Serialize;
use std::{
    collections::HashMap,
    time::{Duration, UNIX_EPOCH},
};
use tokio::process::Command;

use bollard::secret::ContainerSummary;

use super::container_verbose::DockerContainerVerbose;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DockerContainerSummary {
    pub id: String,
    pub image_id: String,
    pub image: String,
    pub command: String,
    pub created: String,
    pub status: String,
    pub ports: String,
    pub names: String,
    pub running: bool,
    read_write_size: String,
    root_fs_size: String,
    labels: Option<HashMap<String, String>>,
    network_mode: Option<String>,
}

impl DockerContainerSummary {
    /// Builds a DockerContainer struct from a bollard::...::ContainerSummary instance.
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

        let running = matches!(c.state.clone().unwrap_or_default().as_str(), "running");

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
            image_id: c.image.clone().unwrap_or_default(),
            command: c.command.clone().unwrap_or_default(),
            created: datetime,
            status: c.status.clone().unwrap_or_default(),
            ports,
            names,
            running,
            read_write_size: String::new(),
            root_fs_size: String::new(),
            labels: None,
            network_mode: None,
        }
    }

    /// Lists all containers present on a given docker daemon
    ///
    /// **Note:** While this returns all containers present, it will
    /// return only the minimal set of values (those which aren't marked optional).
    /// To retrieve all possible values, execute the `refresh` method on the instance.
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

    /// Delete the container from the relevant docker daemon
    pub async fn delete(&self, docker: &bollard::Docker, force: bool) -> Result<()> {
        let opt = RemoveContainerOptions {
            force,
            ..Default::default()
        };
        docker.remove_container(&self.id, Some(opt)).await?;
        Ok(())
    }

    /// Start the container on the docker daemon
    pub async fn start(&self, docker: &bollard::Docker) -> Result<()> {
        docker
            .start_container::<String>(&self.id, None)
            .await
            .context("failed to start container")?;

        Ok(())
    }

    /// Stop the container from running
    pub async fn stop(&self, docker: &bollard::Docker) -> Result<()> {
        docker
            .stop_container(&self.id, None)
            .await
            .context("failed to stop container")?;
        Ok(())
    }

    /// Exec into the container with the given command
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

    /// Update the container instance, replacing it with the current
    /// state of the container.  Can be used to retrieve the full set of
    /// details from an instance retrieved by the `list` method.
    pub async fn verbose(self, docker: &bollard::Docker) -> Result<DockerContainerVerbose> {
        DockerContainerVerbose::from(self, docker).await
    }
}

//  /// Add verbose information to a container from a container summary;
//     /// A potentially premature optimisation, however for tables we are only
//     /// interetsed in a small amount of data; we don't necessarily want to keep
//     /// all info about all containers in memory at all times.
//     fn include_verbose_from_container_summary(&mut self, c: ContainerSummary) {
//         let read_write_size = Byte::from_i64(c.size_rw.clone().unwrap_or_default())
//             .unwrap_or(Byte::from_u64(0))
//             .get_appropriate_unit(UnitType::Binary);

//         let root_fs_size = Byte::from_i64(c.size_root_fs.clone().unwrap_or_default())
//             .unwrap_or(Byte::from_u64(0))
//             .get_appropriate_unit(UnitType::Binary);

//         self.read_write_size = format!("{read_write_size:.2}");
//         self.root_fs_size = format!("{root_fs_size:.2}");
//         self.network_mode = match &c.host_config {
//             Some(c) => c.network_mode.clone(),
//             None => None,
//         };
//         self.labels = c.labels.clone();
//     }

use bollard::query_parameters::{
    ListContainersOptionsBuilder, RemoveContainerOptionsBuilder, StartContainerOptions,
    StopContainerOptions,
};
use chrono::Local;
use chrono::prelude::DateTime;
use color_eyre::eyre::{Context, Result, bail};
use itertools::Itertools;
use serde::{Serialize, Serializer};
use std::{
    collections::HashMap,
    time::{Duration, UNIX_EPOCH},
};
use tokio::process::Command;

use bollard::models::{Port as BollardPort, PortTypeEnum as BollardPortTypeEnum};
use bollard::secret::ContainerSummary;

use super::traits::Describe;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DockerContainer {
    pub id: String,
    pub image_id: String,
    pub image: String,
    pub command: String,
    pub created: String,
    pub status: String,
    pub ports: Ports,
    pub names: String,
    pub running: bool,
    read_write_size: String,
    root_fs_size: String,
    labels: Option<HashMap<String, String>>,
    network_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, parse_display::Display)]
pub(crate) enum Ip {
    #[display("*")]
    All,
    #[display("")]
    Localhost,
    #[display("{0}")]
    Net(String),
}

#[derive(Debug, Clone, PartialEq, Eq, parse_display::Display)]
pub(crate) enum PortType {
    Tcp,
    Udp,
    Sctp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Port {
    ip: Ip,
    private_port: u16,
    public_port: Option<u16>,
    port_type: Option<PortType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Ports(Vec<Port>);

impl DockerContainer {
    /// Builds a DockerContainer struct from a bollard::...::ContainerSummary instance.
    pub fn from(c: ContainerSummary) -> Self {
        let ports = c
            .ports
            .map(|ports| Ports(ports.into_iter().map_into().collect()))
            .unwrap_or_default();
        let datetime = DateTime::<Local>::from(
            UNIX_EPOCH
                + Duration::from_secs(c.created.unwrap_or_default().try_into().unwrap_or_default()),
        )
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

        let running = matches!(c.state.as_ref(), Some(state) if state.to_string().to_lowercase() == "running");

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
    pub async fn list(docker: &bollard::Docker) -> Result<Vec<Self>> {
        let opts = ListContainersOptionsBuilder::default().all(true).build();
        let containers = docker
            .list_containers(Some(opts))
            .await
            .context("unable to retrieve list of containers")?
            .into_iter()
            .map(Self::from)
            .collect();

        Ok(containers)
    }

    /// Delete the container from the relevant docker daemon
    pub async fn delete(&self, docker: &bollard::Docker, force: bool) -> Result<()> {
        let opt = RemoveContainerOptionsBuilder::default()
            .force(force)
            .build();
        docker.remove_container(&self.id, Some(opt)).await?;
        Ok(())
    }

    /// Start the container on the docker daemon
    pub async fn start(&self, docker: &bollard::Docker) -> Result<()> {
        let opts = StartContainerOptions::default();
        docker
            .start_container(&self.id, Some(opts))
            .await
            .context("unable to start container")?;

        Ok(())
    }

    /// Stop the container from running
    pub async fn stop(&self, docker: &bollard::Docker) -> Result<()> {
        let opts = StopContainerOptions::default();
        docker
            .stop_container(&self.id, Some(opts))
            .await
            .context("unable to stop container")?;
        Ok(())
    }

    /// Exec into the container with the given command
    pub async fn attach(&self, cmd: &str) -> Result<()> {
        Command::new("clear").spawn()?.wait().await?;

        let parts: Vec<String> = cmd.split_whitespace().map(String::from).collect();

        let mut command = Command::new("docker");

        let mut arged_commands = command.arg("exec").arg("-it").arg(&self.names);

        for part in parts {
            arged_commands = arged_commands.arg(part);
        }

        let exit_status = arged_commands.spawn()?.wait().await?;

        Command::new("clear").spawn()?.wait().await?;

        if !exit_status.success() {
            bail!("error in connecting to or interacting with container")
        }

        Ok(())
    }
}

impl Describe for DockerContainer {
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        format!("container: {}", self.names)
    }
    fn describe(&self) -> Result<Vec<String>> {
        let summary = match serde_yml::to_string(&self) {
            Ok(s) => s,
            Err(_) => {
                bail!("failed to parse container summary")
            }
        };
        Ok(summary.lines().map(String::from).collect())
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let public_port = self.public_port.map(|p| p.to_string()).unwrap_or_default();
        let port_type = self
            .port_type
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_default();
        write!(
            f,
            "{}:{public_port}:{}:{port_type}",
            self.ip, self.private_port,
        )
    }
}

impl std::fmt::Display for Ports {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl From<BollardPort> for Port {
    fn from(port: BollardPort) -> Self {
        let ip = match port.ip {
            None => Ip::Localhost,
            Some(s) if s.starts_with("127.0.0") => Ip::Localhost,
            Some(s) if s == "0.0.0.0" => Ip::All,
            Some(s) => Ip::Net(s),
        };
        let port_type = match port.typ {
            Some(BollardPortTypeEnum::TCP) => Some(PortType::Tcp),
            Some(BollardPortTypeEnum::UDP) => Some(PortType::Udp),
            Some(BollardPortTypeEnum::SCTP) => Some(PortType::Sctp),
            Some(BollardPortTypeEnum::EMPTY) | None => None,
        };
        Self {
            ip,
            private_port: port.private_port,
            public_port: port.public_port,
            port_type,
        }
    }
}

impl Serialize for Ports {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let ports = self.0.iter().map(|p| p.to_string()).join(", ");
        serializer.serialize_str(&ports)
    }
}

impl Ord for Ports {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl Ord for Port {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.public_port
            .unwrap_or(self.private_port)
            .cmp(other.public_port.as_ref().unwrap_or(&other.private_port))
    }
}

impl PartialOrd for Ports {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd for Port {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use bollard::models::{Port, PortTypeEnum};

//     fn create_mock_container_summary() -> ContainerSummary {
//         ContainerSummary {
//             id: Some("abc123".to_string()),
//             names: Some(vec!["/test-container".to_string()]),
//             image: Some("test-image:latest".to_string()),
//             image_id: Some("sha256:def456".to_string()),
//             command: Some("/bin/bash".to_string()),
//             created: Some(1234567890),
//             ports: Some(vec![Port {
//                 ip: Some("0.0.0.0".to_string()),
//                 private_port: 8080,
//                 public_port: Some(80),
//                 typ: Some(PortTypeEnum::TCP),
//             }]),
//             state: Some("running".to_string()),
//             status: Some("Up 2 hours".to_string()),
//             ..Default::default()
//         }
//     }

//     #[test]
//     fn test_container_from_summary() {
//         let summary = create_mock_container_summary();
//         let container = DockerContainer::from(summary.clone());

//         assert_eq!(container.id, summary.id.unwrap());
//         assert_eq!(container.image, summary.image.unwrap());
//         assert_eq!(container.command, summary.command.unwrap());
//         assert!(container.running);
//         assert_eq!(container.status, summary.status.unwrap());
//         assert_eq!(container.names, "test-container");
//         assert!(container.ports.contains("0.0.0.0:8080:80:TCP"));
//     }

//     #[test]
//     fn test_container_from_summary_minimal() {
//         let summary = ContainerSummary {
//             id: Some("abc123".to_string()),
//             ..Default::default()
//         };
//         let container = DockerContainer::from(summary);

//         assert_eq!(container.id, "abc123");
//         assert_eq!(container.image, "");
//         assert_eq!(container.command, "");
//         assert!(!container.running);
//         assert_eq!(container.status, "");
//         assert_eq!(container.names, "");
//         assert_eq!(container.ports, "");
//     }

//     #[test]
//     fn test_container_describe() {
//         let container = DockerContainer {
//             id: "abc123".to_string(),
//             image: "test-image:latest".to_string(),
//             image_id: "sha256:def456".to_string(),
//             command: "/bin/bash".to_string(),
//             created: "2024-01-01 12:00:00".to_string(),
//             status: "Up 2 hours".to_string(),
//             ports: "80:8080".to_string(),
//             names: "test-container".to_string(),
//             running: true,
//             read_write_size: "".to_string(),
//             root_fs_size: "".to_string(),
//             labels: None,
//             network_mode: None,
//         };

//         let description = container.describe().unwrap();
//         assert!(!description.is_empty());

//         // Verify key information is present in the description
//         let desc_str = description.join("\n");
//         assert!(desc_str.contains(&container.id));
//         assert!(desc_str.contains(&container.image));
//         assert!(desc_str.contains(&container.command));
//     }
// }

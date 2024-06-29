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

use super::container_summary::DockerContainerSummary;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DockerContainerVerbose {
    // pub id: String,
    // pub image_id: String,
    // pub image: String,
    // pub command: String,
    // pub created: String,
    // pub status: String,
    // pub ports: String,
    // pub names: String,
    // pub running: bool,
    // read_write_size: String,
    // root_fs_size: String,
    // labels: Option<HashMap<String, String>>,
    // network_mode: Option<String>,
}

impl DockerContainerVerbose {
    pub async fn from(c: DockerContainerSummary, docker: &bollard::Docker) -> Result<Self> {
        let c: bollard::secret::ContainerInspectResponse = docker
            .inspect_container(&c.id, Some(InspectContainerOptions { size: true }))
            .await?;
        Ok(Self {})
    }

    pub fn to_summary(self) -> DockerContainerSummary {}
}

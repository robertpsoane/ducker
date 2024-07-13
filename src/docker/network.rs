use bollard::secret::{Network, NetworkContainer};
use color_eyre::eyre::{bail, Result};
use serde::Serialize;
use std::collections::HashMap;

use super::traits::Describe;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DockerNetwork {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub created_at: String,
    pub scope: String,
    pub internal: Option<bool>,
    pub attachable: Option<bool>,
    pub containers: Option<HashMap<String, NetworkContainer>>,
}

impl DockerNetwork {
    pub fn from(v: Network) -> Self {
        Self {
            id: v.id.unwrap_or_default(),
            name: v.name.unwrap_or_default(),
            driver: v.driver.unwrap_or_default(),
            created_at: v.created.unwrap_or_default(),
            scope: v.scope.unwrap_or_default(),
            internal: v.internal,
            attachable: v.attachable,
            containers: v.containers,
        }
    }

    pub async fn list(docker: &bollard::Docker) -> Result<Vec<Self>> {
        let networks = docker.list_networks::<String>(None).await?;
        let mut network: Vec<Self> = networks.into_iter().map(Self::from).collect();

        network.sort_by_key(|v| v.name.clone());

        Ok(network)
    }

    pub async fn delete(&self, docker: &bollard::Docker) -> Result<()> {
        docker.remove_network(&self.get_name()).await?;
        Ok(())
    }
}

impl Describe for DockerNetwork {
    fn get_id(&self) -> String {
        self.get_name()
    }
    fn get_name(&self) -> String {
        self.name.clone()
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

use bollard::query_parameters::ListNetworksOptionsBuilder;
use bollard::secret::{Network, NetworkContainer};
use color_eyre::eyre::Result;
use serde::Serialize;
use std::collections::HashMap;

use crate::docker::traits::DescribeSection;

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
        let opts = ListNetworksOptionsBuilder::default().build();
        let networks = docker.list_networks(Some(opts)).await?;
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

    fn describe(&self) -> Result<Vec<DescribeSection>> {
        let mut summary = DescribeSection::new("Summary");
        summary
            .item("ID", &self.id)
            .item("Name", &self.name)
            .item("Driver", &self.driver)
            .item("Created At", &self.created_at)
            .item("Scope", &self.scope)
            .item(
                "Internal",
                self.internal.map(|v| v.to_string()).unwrap_or("N/A".into()),
            )
            .item(
                "Attachable",
                self.attachable
                    .map(|v| v.to_string())
                    .unwrap_or("N/A".into()),
            )
            .item(
                "Containers",
                self.containers
                    .as_ref()
                    .map(|c| c.len().to_string())
                    .unwrap_or("0".into()),
            );
        Ok(vec![summary])
    }
}

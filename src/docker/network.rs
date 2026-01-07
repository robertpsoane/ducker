use bollard::query_parameters::ListNetworksOptionsBuilder;
use bollard::secret::Network;
use color_eyre::eyre::Result;

use crate::docker::traits::DescribeSection;

use super::traits::Describe;

#[derive(Debug, Clone, PartialEq)]
pub struct DockerNetwork {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub created_at: String,
    pub scope: String,
    pub internal: Option<bool>,
    pub attachable: Option<bool>,
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
            );
        Ok(vec![summary])
    }
}

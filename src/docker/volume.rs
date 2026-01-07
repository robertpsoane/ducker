use bollard::query_parameters::{ListVolumesOptionsBuilder, RemoveVolumeOptionsBuilder};
use bollard::secret::{Volume, VolumeScopeEnum};
use byte_unit::{Byte, UnitType};
use color_eyre::eyre::{Result, bail};
use std::collections::HashMap;

use crate::docker::traits::DescribeSection;

use super::traits::Describe;

#[derive(Debug, Clone, PartialEq)]
pub struct DockerVolume {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created_at: Option<String>,
    pub labels: HashMap<String, String>,
    pub scope: Option<VolumeScopeEnum>,
    pub options: HashMap<String, String>,
    pub ref_count: Option<u64>,
    pub size: Option<String>,
}

impl DockerVolume {
    pub fn from(v: Volume) -> Self {
        let ref_count: Option<u64>;
        let size: Option<String>;

        if let Some(usage_data) = v.usage_data {
            if usage_data.ref_count < 0 {
                ref_count = None
            } else {
                ref_count = Some(usage_data.ref_count as u64)
            }

            if usage_data.size < 0 {
                size = None
            } else {
                let byte =
                    Byte::from_u64(usage_data.size as u64).get_appropriate_unit(UnitType::Binary);
                size = Some(format!("{byte:.2}"));
            }
        } else {
            ref_count = None;
            size = None;
        }

        Self {
            name: v.name,
            driver: v.driver,
            mountpoint: v.mountpoint,
            created_at: v.created_at,
            labels: v.labels,
            scope: v.scope,
            options: v.options,
            ref_count,
            size,
        }
    }

    pub async fn list(docker: &bollard::Docker) -> Result<Vec<Self>> {
        let opts = ListVolumesOptionsBuilder::default().build();
        let bollard_volumes = docker.list_volumes(Some(opts)).await?;
        let mut docker_volumes: Vec<Self> = match bollard_volumes.volumes {
            Some(v) => v,
            None => bail!(""),
        }
        .into_iter()
        .map(Self::from)
        .collect();

        docker_volumes.sort_by_key(|v| v.name.clone());

        Ok(docker_volumes)
    }

    pub async fn delete(&self, docker: &bollard::Docker, force: bool) -> Result<()> {
        let opts = RemoveVolumeOptionsBuilder::default().force(force).build();
        docker.remove_volume(&self.get_name(), Some(opts)).await?;
        Ok(())
    }
}

impl Describe for DockerVolume {
    fn get_id(&self) -> String {
        self.get_name()
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn describe(&self) -> Result<Vec<DescribeSection>> {
        let mut summary = DescribeSection::new("Summary");
        summary
            .item("Name", &self.name)
            .item("Driver", &self.driver)
            .item("Mountpoint", &self.mountpoint)
            .item_opt("Created At", self.created_at.as_ref())
            .item_opt("Scope", self.scope)
            .item_opt("Reference Count", self.ref_count)
            .item_opt("Size", self.size.as_ref());
        let mut result = vec![summary];

        if !self.labels.is_empty() {
            let mut label_section = DescribeSection::new("Labels");
            for (key, value) in &self.labels {
                label_section.item(key, value);
            }
            result.push(label_section);
        }
        if !self.options.is_empty() {
            let mut options_section = DescribeSection::new("Options");
            for (key, value) in &self.options {
                options_section.item(key, value);
            }
            result.push(options_section);
        }

        Ok(result)
    }
}

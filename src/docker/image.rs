use byte_unit::{Byte, UnitType};
use chrono::prelude::DateTime;
use chrono::Local;
use color_eyre::eyre::{Context, Result};
use itertools::Itertools;
use std::collections::HashMap;
use std::time::{Duration, UNIX_EPOCH};

use bollard::{image::ListImagesOptions, secret::ImageSummary};

#[derive(Debug, Clone)]
pub struct DockerImage {
    pub id: String,
    pub name: String,
    pub tag: String,
    pub created: String,
    pub size: String,
}

impl DockerImage {
    pub fn from(bollard_image: ImageSummary) -> Vec<Self> {
        let mut response = vec![];

        let datetime = DateTime::<Local>::from(
            UNIX_EPOCH + Duration::from_secs(bollard_image.created.try_into().unwrap_or_default()),
        )
        .format("%Y-%m-%d %H:%M:%S");
        let b = Byte::from_u64(bollard_image.size as u64).get_appropriate_unit(UnitType::Binary);

        println!("{}", bollard_image.containers);

        for repo_tag in bollard_image.repo_tags {
            let split_tag = repo_tag.split(':').collect::<Vec<&str>>();

            response.push(Self {
                id: bollard_image.id.clone(),
                name: split_tag[0].to_string(),
                tag: split_tag[1].to_string(),
                created: datetime.to_string(),
                size: format!("{b:.2}"),
            })
        }
        response
    }

    pub async fn list(docker: &bollard::Docker) -> Result<Vec<Self>> {
        let mut filters: HashMap<String, Vec<String>> = HashMap::new();
        filters.insert("dangling".into(), vec!["false".into()]);

        let mut images = docker
            .list_images(Some(ListImagesOptions::<String> {
                all: false,
                digests: false,
                filters,
            }))
            .await
            .context("unable to retrieve list of images")?
            .into_iter()
            .flat_map(DockerImage::from)
            .collect_vec();
        images.sort_by_key(|i| i.id.clone());
        Ok(images)
    }

    pub async fn delete(&self, docker: &bollard::Docker) -> Result<()> {
        docker
            .remove_image(&self.get_full_name(), None, None)
            .await?;
        Ok(())
    }

    pub fn get_full_name(&self) -> String {
        format!("{}:{}", self.name, self.tag)
    }
}

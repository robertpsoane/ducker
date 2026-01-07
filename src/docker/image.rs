use bollard::query_parameters::{ListImagesOptionsBuilder, RemoveImageOptionsBuilder};
use byte_unit::{Byte, UnitType};
use chrono::Local;
use chrono::prelude::DateTime;
use color_eyre::eyre::{Context, Result};
use itertools::Itertools;
use std::collections::HashMap;
use std::time::{Duration, UNIX_EPOCH};

use bollard::secret::ImageSummary;

use crate::docker::traits::DescribeSection;

use super::traits::Describe;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DockerImage {
    pub id: String,
    pub name: String,
    pub tag: String,
    pub created: String,
    pub size: String,
    pub tags: Vec<String>,
    pub digests: Vec<String>,
}

impl DockerImage {
    pub fn from(bollard_image: ImageSummary) -> Vec<Self> {
        let mut response = vec![];

        let datetime = DateTime::<Local>::from(
            UNIX_EPOCH + Duration::from_secs(bollard_image.created.try_into().unwrap_or_default()),
        )
        .format("%Y-%m-%d %H:%M:%S");
        let b = Byte::from_u64(bollard_image.size as u64).get_appropriate_unit(UnitType::Binary);

        let tags = bollard_image.repo_tags.clone();
        let digests = bollard_image.repo_digests.clone();

        if !bollard_image.repo_tags.is_empty() {
            for repo_tag in bollard_image.repo_tags {
                let split_tag = repo_tag.split(':').collect::<Vec<&str>>();

                response.push(Self {
                    id: bollard_image.id.clone(),
                    name: split_tag[0].to_string(),
                    tag: split_tag[1].to_string(),
                    created: datetime.to_string(),
                    size: format!("{b:.2}"),
                    tags: tags.clone(),
                    digests: digests.clone(),
                })
            }
        } else {
            response.push(Self {
                id: bollard_image.id.clone(),
                name: "<none>".into(),
                tag: "<none>".into(),
                created: datetime.to_string(),
                size: format!("{b:.2}"),
                tags,
                digests,
            })
        }
        response
    }

    pub async fn list(docker: &bollard::Docker, dangling: bool) -> Result<Vec<Self>> {
        let mut filters: HashMap<String, Vec<String>> = HashMap::new();
        if !dangling {
            filters.insert("dangling".into(), vec!["false".into()]);
        }

        let opts = ListImagesOptionsBuilder::default()
            .all(true)
            .digests(false)
            .filters(&filters)
            .build();
        let mut images = docker
            .list_images(Some(opts))
            .await
            .context("unable to retrieve list of images")?
            .into_iter()
            .flat_map(DockerImage::from)
            .collect_vec();
        images.sort_by_key(|i| i.id.clone());
        Ok(images)
    }

    pub async fn delete(&self, docker: &bollard::Docker, force: bool) -> Result<()> {
        let opts = RemoveImageOptionsBuilder::default().force(force).build();
        docker
            .remove_image(&self.get_full_name(), Some(opts), None)
            .await?;
        Ok(())
    }

    pub fn get_full_name(&self) -> String {
        let image = format!("{}:{}", self.name, self.tag);

        if image == "<none>:<none>" {
            self.id.clone()
        } else {
            image
        }
    }
}

impl Describe for DockerImage {
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_name(&self) -> String {
        format!("image: {}", self.name)
    }
    fn describe(&self) -> Result<Vec<DescribeSection>> {
        let mut summary = DescribeSection::new("Summary");
        summary
            .item("ID", &self.id)
            .item("Name", &self.name)
            .item("Tag", &self.tag)
            .item("Created", &self.created)
            .item("Size", &self.size)
            .item("Tags", self.tags.join(", "))
            .item("Digests", self.digests.join(", "));
        Ok(vec![summary])
    }
}

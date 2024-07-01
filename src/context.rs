use crate::docker::{container_summary::DockerContainerSummary, image::DockerImage};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AppContext {
    pub list_idx: Option<usize>,
    pub docker_container: Option<DockerContainerSummary>,
    pub docker_image: Option<DockerImage>,
}

impl AppContext {}

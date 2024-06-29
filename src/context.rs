use crate::docker::{container::DockerContainer, image::DockerImage};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AppContext {
    pub list_idx: Option<usize>,
    pub docker_container: Option<DockerContainer>,
    pub docker_image: Option<DockerImage>,
}

impl AppContext {}

use crate::{
    docker::{container::DockerContainer, image::DockerImage, traits::Describe},
    events::Transition,
};

/// AppContext is used to share context between pages
/// Includes a set of optional fields that can be sent to the
/// next page.
/// Where `then` is set, the next page should use that as the
/// transition on completion instead of any other transition
#[derive(Clone, Debug, Default)]
pub struct AppContext {
    pub then: Option<Box<Transition>>,
    pub list_idx: Option<usize>,
    pub docker_container: Option<DockerContainer>,
    pub docker_image: Option<DockerImage>,
    pub describable: Option<Box<dyn Describe>>,
}

impl AppContext {
    pub fn next(&self) -> Option<Transition> {
        self.then.as_ref().map(|t| *t.clone())
    }
}

impl PartialEq for AppContext {
    fn eq(&self, other: &Self) -> bool {
        if self.list_idx != other.list_idx {
            return false;
        }

        if self.docker_container != other.docker_container {
            return false;
        }

        if self.docker_image != other.docker_image {
            return false;
        }

        // Describe doesn't have derived PartialEqual trait
        // We can assume that if both offer the same description,
        // then they are equal
        if (self.describable.is_some() && other.describable.is_none())
            || (self.describable.is_none() && other.describable.is_some())
        {
            return false;
        }

        if self.describable.is_some() && other.describable.is_some() {
            match (
                self.describable.clone().unwrap().describe(),
                other.describable.clone().unwrap().describe(),
            ) {
                (Ok(s), Ok(o)) => return s == o,
                (Err(s), Err(o)) => return format!("{s}") == format!("{o}"),
                (_, _) => return false,
            }
        }

        true
    }
}

impl AppContext {}

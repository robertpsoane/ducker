use std::fmt;

use color_eyre::eyre::Result;
use dyn_clone::DynClone;
use uuid::Uuid;

/// Provides an interface to describe the contents of the implementing
/// struct in a human readable format.
/// Provides a generic minimal description interface over a selection of
/// docker resources
pub trait Describe: fmt::Debug + Send + Sync + DynClone {
    /// Get the ID of the resource being described
    fn get_id(&self) -> String;
    /// Get a human readable name of the resource being described
    fn get_name(&self) -> String;
    /// Get a human readable description of the resource being described
    fn describe(&self) -> Result<Vec<DescribeSection>>;
}

dyn_clone::clone_trait_object!(Describe);

#[derive(Debug, PartialEq, Eq)]
pub struct DescribeSection {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) items: Vec<DescribeItem>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DescribeItem {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) value: String,
}

impl DescribeItem {
    pub fn new<N: ToString, V: ToString>(name: N, value: V) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

impl DescribeSection {
    pub fn new<N: ToString>(name: N) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            items: vec![],
        }
    }

    pub(crate) fn item<N: ToString, V: ToString>(&mut self, name: N, value: V) -> &mut Self {
        self.items.push(DescribeItem::new(name, value));
        self
    }
}

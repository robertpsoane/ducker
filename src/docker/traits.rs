use std::fmt;

use color_eyre::eyre::Result;
use dyn_clone::DynClone;

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
    fn describe(&self) -> Result<Vec<String>>;
}

dyn_clone::clone_trait_object!(Describe);

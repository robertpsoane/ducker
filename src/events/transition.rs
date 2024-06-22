use crate::docker::container::DockerContainer;

/// A transition is a type of event that flows "in reverse" when compared
/// with input events.  A transition can be emitted from a component
/// The transition will then be handled at a higher level (eg in the app
/// or the page manager)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transition {
    Quit,
    ToNewTerminal,
    ToViewMode,
    ToImagePage,
    ToContainerPage,
    ToLogPage(DockerContainer),
    ToAttach(DockerContainer),
}

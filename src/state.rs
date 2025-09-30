// TODO: Merge mode and running to State { View, TextInput, Finishing ... }
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    View,
    TextInput,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Running {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum CurrentPage {
    #[default]
    Containers,
    Images,
    Volumes,
    Logs,
    Attach,
    Network,
    DescribeContainer,
    Help,
}

// impl Default for CurrentPage {
//     fn default() -> Self {
//         Self::Containers(AppContext::default())
//     }
// }

use color_eyre::eyre::{Context, Result};
use tokio::sync::mpsc::Sender;

use crate::context::AppContext;
use crate::events::{Key, Message};

/// A transition is a type of event that flows "in reverse" when compared
/// with input events.  A transition can be emitted from a component
/// The transition will then be handled at a higher level (eg in the app
/// or the page manager)
#[derive(Debug, Clone, PartialEq)]
pub enum Transition {
    Quit,
    ToNewTerminal,
    ToViewMode,
    ToImagePage(AppContext),
    ToContainerPage(AppContext),
    ToLogPage(AppContext),
    ToDescribeContainerPage(AppContext),
    ToAttach(AppContext),
    ToVolumePage(AppContext),
    ToNetworkPage(AppContext),
}

pub async fn send_transition(
    tx: Sender<Message<Key, Transition>>,
    transition: Transition,
) -> Result<()> {
    tx.send(Message::Transition(transition))
        .await
        .context("unable to send transition")?;
    Ok(())
}

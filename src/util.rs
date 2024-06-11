use color_eyre::eyre::{Context, Result};
use tokio::sync::mpsc::Sender;

use crate::events::{Key, Message, Transition};

pub async fn send_transition(
    tx: Sender<Message<Key, Transition>>,
    transition: Transition,
) -> Result<()> {
    tx.send(Message::Transition(transition))
        .await
        .context("unable to send transition")?;
    Ok(())
}

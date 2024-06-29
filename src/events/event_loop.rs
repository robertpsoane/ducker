use color_eyre::eyre::ContextCompat;
use color_eyre::eyre::Result;
use crossterm::event::Event as CrossTermEvent;
use futures::lock::Mutex;
use futures::{FutureExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::{
    sync::mpsc::{self},
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
    time::interval,
};

use super::key::Key;
use super::Message;
use super::Transition;

const TICK_RATE: Duration = Duration::from_millis(250);

pub struct EventLoop {
    /// Outbound sender; used to send internal messages to outbound receiver
    outbound_tx: Sender<Message<Key, Transition>>,
    /// Outbound sender; used to receive messages and pass them out of the event loop
    outbound_rx: Receiver<Message<Key, Transition>>,

    /// Inbound sender; used to send messages into eventloop from outside
    inbound_tx: Sender<Message<Key, Transition>>,
    /// Inbound sender; used to receive external messages and proxy them to inbound receiver
    inbound_rx: Arc<Mutex<Receiver<Message<Key, Transition>>>>,
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl EventLoop {
    pub fn new() -> Self {
        let (outbound_tx, outbound_rx) = mpsc::channel::<Message<Key, Transition>>(32);
        let (inbound_tx, inbound_rx) = mpsc::channel::<Message<Key, Transition>>(32);
        let mutexed_inbound_rx = Arc::new(Mutex::new(inbound_rx));
        Self {
            outbound_tx,
            outbound_rx,
            inbound_tx,
            inbound_rx: mutexed_inbound_rx,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.spawn_tick_task();
        self.spawn_io_task();
        self.spawn_inbound_task();
        Ok(())
    }

    pub async fn next(&mut self) -> Result<Message<Key, Transition>> {
        let event = self
            .outbound_rx
            .recv()
            .await
            .context("unable to receive event")?;
        Ok(event)
    }

    pub fn get_tx(&self) -> Sender<Message<Key, Transition>> {
        self.inbound_tx.clone()
    }

    fn spawn_tick_task(&self) -> JoinHandle<()> {
        let tx = self.outbound_tx.clone();
        let mut interval = interval(TICK_RATE);
        tokio::spawn(async move {
            loop {
                let delay = interval.tick();
                tokio::select! {
                    _ = tx.closed() => {
                        break;
                    }
                    _ = delay =>  {
                        tx.send(Message::Tick).await.unwrap();
                    }

                }
            }
        })
    }

    fn spawn_io_task(&self) -> JoinHandle<()> {
        let tx = self.outbound_tx.clone();
        tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = interval(TICK_RATE);
            loop {
                let delay = tick.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    _ = tx.closed() => {
                        break;
                    }
                    _ = delay => {}
                    Some(Ok(event)) = crossterm_event => {
                        if let CrossTermEvent::Key(key) = event {
                            let key = Key::from(key);
                            tx.send(Message::Input(key)).await.unwrap();
                        }
                    }
                }
            }
        })
    }

    fn spawn_inbound_task(&self) -> JoinHandle<()> {
        let tx = self.outbound_tx.clone();
        let rx = self.inbound_rx.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tx.closed() => {
                        break;
                    }
                    mut locked_rx = rx.lock() => {
                        if let Some(event) = locked_rx.recv().await {
                            tx.send(event).await.unwrap()
                        }
                    }
                }
            }
        })
    }
}

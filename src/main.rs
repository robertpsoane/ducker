use clap::Parser;
use color_eyre::eyre::Context;
use events::{EventLoop, Key, Message};
use ui::App;

mod autocomplete;
mod state;
mod util;
mod ui {
    pub mod app;
    pub mod page_manager;

    pub use app::App;
}
mod traits {
    mod component;
    mod page;

    pub use component::Component;
    pub use page::Page;
}
mod events {
    pub mod event_loop;
    pub mod key;
    pub mod message;
    pub mod transition;

    pub use event_loop::EventLoop;
    pub use key::Key;
    pub use message::Message;
    pub use transition::Transition;
}
mod docker {
    pub mod container;
    pub mod image;
}
mod pages {
    pub mod containers;
    pub mod images;
}
mod components {
    pub mod confirmation_modal;
    pub mod footer;
    pub mod header;
    pub mod help;
    pub mod input_field;
}
pub mod terminal;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = String::from("unix:///var/run/docker.sock"))]
    docker_daemon: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    terminal::init_panic_hook();
    let mut terminal = terminal::init().context("failed to initialise terminal")?;

    let mut events = EventLoop::new();
    let events_tx = events.get_tx().await;
    let mut app = App::new(events_tx).await.context("failed to create app")?;

    events.start().context("failed to start event loop")?;

    while app.running != state::Running::Done {
        terminal
            .draw(|f| {
                app.draw(f);
            })
            .context("failed to update view")?;

        match events
            .next()
            .await
            .context("unable to receive next event")?
        {
            Message::Input(k) => {
                let res = app.update(k).await.context("failed to update")?;
                if !res.is_consumed() {
                    // If in system quit events
                    if k == Key::Ctrl('c') || k == Key::Ctrl('d') {
                        break;
                    }
                }
            }
            Message::Transition(t) => {
                let _ = &app
                    .transition(t)
                    .await
                    .context("unable to execute transition")?;
            }

            Message::Tick => {
                app.update(Key::Null).await.context("failed to update")?;
            }
        }
    }

    terminal::restore().context("failed to restore terminal")?;

    Ok(())
}

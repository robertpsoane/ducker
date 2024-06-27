use clap::Parser;
use color_eyre::eyre::Context;
use events::{EventLoop, Key, Message};
use ui::App;

mod autocomplete;
mod context;
mod state;
mod util;
mod ui {
    pub mod app;
    pub mod page_manager;

    pub use app::App;
}
mod callbacks {
    pub mod delete_container;
    pub mod delete_image;

    pub use delete_container::DeleteContainer;
}
mod traits {
    mod callback;
    mod component;
    mod page;

    pub use callback::Callback;
    pub use component::Component;
    pub use component::ModalComponent;
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
mod widgets {
    pub mod modal;
}
mod docker {
    pub mod container;
    pub mod image;
    pub mod logs;
}
mod pages {
    pub mod attach;
    pub mod containers;
    pub mod images;
    pub mod logs;
}
mod components {
    pub mod alert_modal;
    pub mod boolean_modal;
    pub mod footer;
    pub mod header;
    pub mod help;
    pub mod input_field;
    pub mod resize_notice;
}
pub mod terminal;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    Args::parse();

    terminal::init_panic_hook();
    let mut terminal = terminal::init().context("failed to initialise terminal")?;

    let mut events = EventLoop::new();
    let events_tx = events.get_tx();
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
                let res = app.update(k).await;
                if !res.is_consumed() {
                    // If in system quit events
                    if k == Key::Ctrl('c') || k == Key::Ctrl('d') {
                        break;
                    }
                }
            }
            Message::Transition(t) => {
                if t == events::Transition::ToNewTerminal {
                    terminal = terminal::init().context("failed to initialise terminal")?;
                } else {
                    let _ = &app.transition(t).await;
                }
            }

            Message::Tick => {
                app.update(Key::Null).await;
            }
        }
    }

    terminal::restore().context("failed to restore terminal")?;

    Ok(())
}

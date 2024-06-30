use clap::Parser;
use color_eyre::eyre::Context;
use config::Config;
use docker::util::new_local_docker_connection;
use events::{EventLoop, Key, Message};
use ui::App;

mod autocomplete;
mod callbacks;
mod components;
mod config;
mod context;
mod docker;
mod events;
mod pages;
mod state;
mod terminal;
mod traits;
mod ui;
mod widgets;

const CONFIGURATION_DOC_PATH: &str = "https://github.com/robertpsoane/ducker#configuration";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Export default config to default config directory
    /// (usually ~/.config/ducker/config.yaml)
    #[clap(long, short, action)]
    export_default_config: bool,

    /// Path at which to find the socket to communicate with
    /// docker
    #[clap(long, short)]
    docker_path: Option<String>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let args = Args::parse();
    let config = Config::new(&args.export_default_config, args.docker_path)?;

    let docker = new_local_docker_connection(&config.docker_path)
        .await
        .context(format!("failed to create docker connection, potentially due to misconfiguration (see {CONFIGURATION_DOC_PATH})"))?;

    terminal::init_panic_hook();

    let mut terminal = terminal::init().context("failed to initialise terminal")?;

    let mut events = EventLoop::new();
    let events_tx = events.get_tx();
    let mut app = App::new(events_tx, docker, config)
        .await
        .context("failed to create app")?;

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

            Message::Error(_) => {
                // This needs implementing
            }
        }
    }

    terminal::restore().context("failed to restore terminal")?;

    Ok(())
}

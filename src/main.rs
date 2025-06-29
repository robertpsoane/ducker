use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Context;

use ducker::{
    config::Config,
    docker::util::new_local_docker_connection,
    events::{self, EventLoop, Key, Message},
    state, terminal,
    tracing::initialize_logging,
    ui::App,
};

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

    /// Docker host URL (e.g. tcp://1.2.3.4:2375)
    /// Overrides DOCKER_HOST environment variable
    #[clap(long)]
    docker_host: Option<String>,

    /// Path at which to write log messages; intended mainly for debugging.
    /// If unset will log to default log location; usually
    /// ~/.local/share/ducker/ducker.log
    ///
    /// Set log level by setting the `DUCKER_LOGLEVEL` environment variable
    /// defaults to `info`.
    #[clap(long, short)]
    log_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let args = Args::parse();
    initialize_logging(&args.log_path).context("failed to initialise logging")?;
    color_eyre::install()?;
    let config = Config::new(
        &args.export_default_config,
        args.docker_path,
        args.docker_host,
    )?;

    let docker = new_local_docker_connection(&config.docker_path, config.docker_host.as_deref())
        .await
        .context(format!("failed to create docker connection, potentially due to misconfiguration (see {CONFIGURATION_DOC_PATH})"))?;
    terminal::init_panic_hook();

    let mut terminal = ratatui::init();
    terminal.clear()?;

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
                    terminal = ratatui::init();
                    terminal.clear()?;
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

    ratatui::restore();

    Ok(())
}

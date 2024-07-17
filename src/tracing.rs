use std::path::PathBuf;

use color_eyre::eyre::Result;
use lazy_static::lazy_static;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::get_app_config_path;

// taken form https://ratatui.rs/recipes/apps/log-with-tracing/

lazy_static! {
    pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
    pub static ref LOG_ENV: String = format!("{}_LOGLEVEL", PROJECT_NAME.clone());
    pub static ref LOG_FILE: String = format!("{}.log", env!("CARGO_PKG_NAME"));
}

fn project_directory() -> Option<PathBuf> {
    dirs_next::data_dir().map(|data_dir| data_dir.join(env!("CARGO_CRATE_NAME")))
}

pub fn get_log_dir() -> PathBuf {
    if let Some(p) = project_directory() {
        p
    } else if let Ok(p) = get_app_config_path() {
        p
    } else {
        PathBuf::from(".").join(".data")
    }
}

pub fn initialize_logging(log_to: &Option<PathBuf>) -> Result<()> {
    let log_path = match log_to {
        Some(p) => p.clone(),
        None => {
            let directory = get_log_dir();
            std::fs::create_dir_all(directory.clone())?;
            directory.join(LOG_FILE.clone())
        }
    };

    let log_file = std::fs::File::create(log_path)?;
    std::env::set_var(
        "RUST_LOG",
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var(LOG_ENV.clone()))
            .unwrap_or_else(|_| format!("{}=info", env!("CARGO_CRATE_NAME"))),
    );
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false);
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    info!("logging initialised");
    Ok(())
}

/// Similar to the `std::dbg!` macro, but generates `tracing` events rather
/// than printing to stdout.
///
/// By default, the verbosity level for the generated events is `DEBUG`, but
/// this can be customized.
#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: $target, $level, ?value, stringify!($ex));
                value
            }
        }
    }};
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };
    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: tracing::Level::DEBUG, $ex)
    };
}

use std::fs;
use std::io::BufReader;
use std::str::FromStr;
use std::{fs::File, path::PathBuf};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use color_eyre::eyre::{bail, Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub prompt: String,

    #[serde(default = "default_exec")]
    pub default_exec: String,

    #[serde(default = "default_docker_path")]
    pub docker_path: String,

    #[serde(default = "default_check_update")]
    pub check_for_update: bool,

    #[serde(default)]
    pub theme: Theme,
}

impl Config {
    pub fn new(write: &bool, docker_path: Option<String>) -> Result<Self> {
        let config_path = get_app_config_path()?.join("config.yaml");
        if *write {
            write_default_config(&config_path).context("failed to write default config")?;
        }

        let mut config: Config;

        if let Ok(f) = File::open(config_path) {
            config = serde_yml::from_reader(BufReader::new(f)).context("unable to parse config")?;
        } else {
            config = Config::default()
        }

        if let Some(p) = docker_path {
            config.docker_path = p;
        }

        Ok(config)
    }
}

fn default_prompt() -> String {
    "🦆".into()
}

fn default_exec() -> String {
    "/bin/bash".into()
}

fn default_docker_path() -> String {
    #[cfg(unix)]
    return "unix:///var/run/docker.sock".into();

    #[cfg(windows)]
    return "npipe:////./pipe/docker_engine".into();
}

fn default_check_update() -> bool {
    true
}

fn default_use_theme() -> bool {
    false
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prompt: default_prompt(),
            default_exec: default_exec(),
            docker_path: default_docker_path(),
            check_for_update: default_check_update(),
            theme: Theme::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Test
    #[serde(default = "default_use_theme")]
    use_theme: bool,

    #[serde(default = "default_title_colour")]
    title: Color,

    #[serde(default = "default_help_colour")]
    help: Color,

    #[serde(default = "default_background_colour")]
    background: Color,

    #[serde(default = "default_success_colour")]
    footer: Color,

    #[serde(default = "default_footer_colour")]
    success: Color,

    #[serde(default = "default_error_colour")]
    error: Color,

    #[serde(default = "default_positive_highlight_colour")]
    positive_highlight: Color,

    #[serde(default = "default_negative_highlight_colour")]
    negative_highlight: Color,
}

impl Theme {
    pub fn title(&self) -> Color {
        if self.use_theme {
            self.title
        } else {
            Color::Green
        }
    }
    pub fn help(&self) -> Color {
        if self.use_theme {
            self.help
        } else {
            Color::Red
        }
    }
    pub fn background(&self) -> Color {
        if self.use_theme {
            self.background
        } else {
            Color::Reset
        }
    }
    pub fn footer(&self) -> Color {
        if self.use_theme {
            self.footer
        } else {
            Color::Cyan
        }
    }
    pub fn success(&self) -> Color {
        if self.use_theme {
            self.success
        } else {
            Color::Green
        }
    }
    pub fn error(&self) -> Color {
        if self.use_theme {
            self.error
        } else {
            Color::Red
        }
    }
    pub fn positive_highlight(&self) -> Color {
        if self.use_theme {
            self.positive_highlight
        } else {
            Color::Green
        }
    }
    pub fn negative_highlight(&self) -> Color {
        if self.use_theme {
            self.negative_highlight
        } else {
            Color::Magenta
        }
    }
}

fn default_title_colour() -> Color {
    Color::from_str("#96e072").unwrap()
}
fn default_success_colour() -> Color {
    Color::from_str("#96e072").unwrap()
}
fn default_help_colour() -> Color {
    Color::from_str("#ee5d43").unwrap()
}
fn default_background_colour() -> Color {
    Color::from_str("#23262e").unwrap()
}
fn default_footer_colour() -> Color {
    Color::from_str("#00e8c6").unwrap()
}
fn default_error_colour() -> Color {
    Color::from_str("#ee5d43").unwrap()
}
fn default_positive_highlight_colour() -> Color {
    Color::from_str("#96e072").unwrap()
}
fn default_negative_highlight_colour() -> Color {
    Color::from_str("#ff00aa").unwrap()
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            use_theme: default_use_theme(),
            title: default_title_colour(),
            help: default_help_colour(),
            success: default_success_colour(),
            background: default_background_colour(),
            footer: default_footer_colour(),
            error: default_error_colour(),
            positive_highlight: default_positive_highlight_colour(),
            negative_highlight: default_negative_highlight_colour(),
        }
    }
}

pub fn get_app_config_path() -> Result<std::path::PathBuf> {
    let path = if cfg!(target_os = "macos") {
        dirs_next::home_dir().map(|h| h.join(".config"))
    } else {
        dirs_next::config_dir()
    };
    if path.is_none() {
        bail!("unable to find config path")
    }
    let mut path = path.unwrap();
    path.push("ducker");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

fn write_default_config(path: &PathBuf) -> Result<()> {
    let config = Config::default();
    fs::write(path, serde_yml::to_string(&config)?)?;
    Ok(())
}

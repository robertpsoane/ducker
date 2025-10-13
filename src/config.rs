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

    #[serde(default)]
    pub docker_host: Option<String>,

    #[serde(default = "default_check_update")]
    pub check_for_update: bool,

    #[serde(default = "default_autocomplete_minimum_length")]
    pub autocomplete_minimum_length: usize,

    #[serde(default)]
    pub theme: Theme,

    #[serde(default)]
    pub format: Option<String>,
}

impl Config {
    pub fn new(
        write: &bool,
        docker_path: Option<String>,
        docker_host: Option<String>,
    ) -> Result<Self> {
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

        if let Some(h) = docker_host {
            config.docker_host = Some(h);
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

fn default_autocomplete_minimum_length() -> usize {
    2
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prompt: default_prompt(),
            default_exec: default_exec(),
            docker_path: default_docker_path(),
            docker_host: None,
            check_for_update: default_check_update(),
            autocomplete_minimum_length: default_autocomplete_minimum_length(),
            theme: Theme::default(),
            format: None,
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

pub fn parse_format_fields(format: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\{\{\.(\w+)\}\}").unwrap();
    let fields: Vec<String> = re.captures_iter(format).map(|cap| cap[1].to_string()).collect();
    if fields.is_empty() {
        vec!["id", "image", "command", "created", "status", "ports", "names"].iter().map(|s| s.to_string()).collect()
    } else {
        fields
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempfile::TempDir;

//     fn create_test_config_dir() -> TempDir {
//         TempDir::new().expect("Failed to create temporary directory")
//     }

//     #[test]
//     fn test_config_new_with_defaults() {
//         let config = Config::new(&false, None, None).unwrap();
//         assert_eq!(config.prompt, "🦆");
//         assert_eq!(config.default_exec, "/bin/bash");
//         #[cfg(unix)]
//         assert_eq!(config.docker_path, "unix:///var/run/docker.sock");
//         #[cfg(windows)]
//         assert_eq!(config.docker_path, "npipe:////./pipe/docker_engine");
//         assert!(config.docker_host.is_none());
//         assert!(config.check_for_update);
//     }

//     #[test]
//     fn test_config_new_with_docker_path() {
//         let custom_path = "/custom/docker.sock";
//         let config = Config::new(&false, Some(custom_path.to_string()), None).unwrap();
//         assert_eq!(config.docker_path, custom_path);
//         assert!(config.docker_host.is_none());
//     }

//     #[test]
//     fn test_config_new_with_docker_host() {
//         let custom_host = "tcp://1.2.3.4:2375";
//         let config = Config::new(&false, None, Some(custom_host.to_string())).unwrap();
//         assert_eq!(config.docker_host, Some(custom_host.to_string()));
//     }

//     #[test]
//     fn test_config_new_with_both_docker_options() {
//         let custom_path = "/custom/docker.sock";
//         let custom_host = "tcp://1.2.3.4:2375";
//         let config = Config::new(
//             &false,
//             Some(custom_path.to_string()),
//             Some(custom_host.to_string()),
//         )
//         .unwrap();
//         assert_eq!(config.docker_path, custom_path);
//         assert_eq!(config.docker_host, Some(custom_host.to_string()));
//     }

//     #[test]
//     fn test_config_write_and_read() {
//         let temp_dir = create_test_config_dir();
//         let config_path = temp_dir.path().join("config.yaml");

//         // Write default config
//         write_default_config(&config_path).unwrap();

//         // Read it back
//         let config: Config =
//             serde_yml::from_reader(BufReader::new(File::open(&config_path).unwrap())).unwrap();

//         // Verify defaults
//         assert_eq!(config.prompt, "🦆");
//         assert_eq!(config.default_exec, "/bin/bash");
//         assert!(config.docker_host.is_none());
//     }

//     #[test]
//     fn test_theme_defaults() {
//         let theme = Theme::default();
//         assert!(!theme.use_theme);
//         // Test that color getters return expected values when use_theme is false
//         assert_eq!(theme.title(), Color::Green);
//         assert_eq!(theme.help(), Color::Red);
//         assert_eq!(theme.background(), Color::Reset);
//         assert_eq!(theme.footer(), Color::Cyan);
//         assert_eq!(theme.success(), Color::Green);
//         assert_eq!(theme.error(), Color::Red);
//         assert_eq!(theme.positive_highlight(), Color::Green);
//         assert_eq!(theme.negative_highlight(), Color::Magenta);
//     }

//     #[test]
//     fn test_theme_custom_colors() {
//         let mut theme = Theme::default();
//         theme.use_theme = true;
//         theme.title = Color::Blue;
//         theme.help = Color::Yellow;

//         // Test that color getters respect custom colors when use_theme is true
//         assert_eq!(theme.title(), Color::Blue);
//         assert_eq!(theme.help(), Color::Yellow);
//     }
// }

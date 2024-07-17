use std::sync::Arc;

use crate::{config::Config, traits::Component};
use color_eyre::eyre::Result;
use ratatui::{
    layout::{Margin, Rect},
    style::Style,
    text::{Line, Span},
    Frame,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const TAGS_URL: &str = "https://api.github.com/repos/robertpsoane/ducker/tags";

#[derive(Debug)]
pub struct VersionComponent {
    config: Arc<Config>,
    version: String,
    update_to: Option<String>,
}

impl VersionComponent {
    pub async fn new(config: Arc<Config>) -> Self {
        let version = format!("v{VERSION}");

        let update_to = if config.check_for_update {
            get_update_to(&version).await
        } else {
            None
        };

        Self {
            config,
            version,
            update_to,
        }
    }
}

impl Component for VersionComponent {
    fn draw(&mut self, f: &mut Frame<'_>, area: Rect) {
        let area = area.inner(Margin {
            vertical: 0,
            horizontal: 1,
        });

        let current_version = Span::from(self.version.clone());

        let spans = if let Some(update) = &self.update_to {
            let update_to_span = Span::from(update);
            let arrow = Span::from(" > ");
            vec![
                current_version.style(Style::default().fg(self.config.theme.negative_highlight())),
                arrow,
                update_to_span.style(Style::default().fg(self.config.theme.positive_highlight())),
            ]
        } else {
            vec![current_version.style(Style::default().fg(self.config.theme.positive_highlight()))]
        };

        f.render_widget(
            Line::from(spans).alignment(ratatui::layout::Alignment::Right),
            area,
        )
    }
}

async fn get_update_to(version: &str) -> Option<String> {
    let latest_version = match find_latest_version().await {
        Ok(v) => v,
        Err(_) => return None,
    };
    if version == latest_version {
        None
    } else {
        Some(latest_version)
    }
}

async fn find_latest_version() -> Result<String> {
    let body: serde_yml::Value = ureq::get(TAGS_URL)
        .set("User-Agent", &format!("Ducker / {VERSION}"))
        .call()?
        .into_json()?;

    let release = match body.get(0) {
        Some(v) => v,
        None => panic!("could not parse response"),
    };

    let release_name = match release.get("name") {
        Some(v) => v,
        None => panic!("could not parse response"),
    };

    let release_name_value = match release_name.as_str() {
        Some(v) => String::from(v),
        None => panic!("could not parse response"),
    };

    Ok(release_name_value)
}

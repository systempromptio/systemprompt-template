use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use serde_json::Value;
use std::path::Path;
use tokio::fs;

pub use super::templates_data::prepare_template_data;
pub use super::templates_navigation::{generate_footer_html, generate_social_action_bar_html};
pub use super::templates_paper::{
    calculate_read_time, generate_toc_html, parse_paper_metadata, render_paper_sections_html,
};

pub async fn load_web_config() -> Result<serde_yaml::Value> {
    let web_config_path = std::env::var("WEB_CONFIG_PATH")
        .unwrap_or_else(|_| "crates/services/web/config.yml".to_string());

    let content = fs::read_to_string(&web_config_path)
        .await
        .map_err(|e| anyhow!("Failed to read web config: {}", e))?;

    serde_yaml::from_str(&content).map_err(|e| anyhow!("Failed to parse web config: {}", e))
}

pub fn get_templates_path(config: &serde_yaml::Value) -> String {
    std::env::var("SCG_TEMPLATES_PATH").unwrap_or_else(|_| {
        config
            .get("paths")
            .and_then(|p| p.get("templates"))
            .and_then(|t| t.as_str())
            .unwrap_or("crates/services/web/templates")
            .to_string()
    })
}

pub fn get_assets_path(config: &serde_yaml::Value) -> String {
    std::env::var("SCG_ASSETS_PATH").unwrap_or_else(|_| {
        config
            .get("paths")
            .and_then(|p| p.get("assets"))
            .and_then(|t| t.as_str())
            .unwrap_or("crates/services/web/assets")
            .to_string()
    })
}

#[derive(Debug)]
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    pub async fn new(template_dir: &str) -> Result<Self> {
        let mut handlebars = Handlebars::new();

        let path = Path::new(template_dir);
        if !path.exists() {
            return Err(anyhow!("Template directory not found: {}", template_dir));
        }

        let mut entries = fs::read_dir(path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "html") {
                let template_name = path.file_stem().unwrap().to_string_lossy().to_string();

                let content = fs::read_to_string(&path).await?;
                handlebars.register_template_string(&template_name, content)?;
            }
        }

        Ok(Self { handlebars })
    }

    pub fn render(&self, template_name: &str, data: &Value) -> Result<String> {
        self.handlebars
            .render(template_name, data)
            .map_err(|e| anyhow!("Template render failed: {}", e))
    }
}

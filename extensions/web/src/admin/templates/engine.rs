use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::Value;

use super::helpers;
use crate::navigation::BrandingConfig;

#[derive(Clone)]
pub struct AdminTemplateEngine {
    hbs: Arc<Handlebars<'static>>,
    branding: Option<BrandingConfig>,
}

impl AdminTemplateEngine {
    pub fn new(admin_dir: &Path) -> Result<Self> {
        let mut hbs = Handlebars::new();
        hbs.set_strict_mode(true);

        let partials_dir = admin_dir.join("partials");
        let templates_dir = admin_dir.join("templates");

        Self::register_partials_recursive(&mut hbs, &partials_dir, &partials_dir)?;
        Self::register_templates(&mut hbs, &templates_dir)?;
        helpers::register_helpers(&mut hbs);

        Ok(Self {
            hbs: Arc::new(hbs),
            branding: None,
        })
    }

    #[must_use]
    pub fn with_branding(mut self, branding: Option<BrandingConfig>) -> Self {
        self.branding = branding;
        self
    }

    #[must_use]
    pub fn branding(&self) -> Option<&BrandingConfig> {
        self.branding.as_ref()
    }

    fn register_partials_recursive(
        hbs: &mut Handlebars<'static>,
        dir: &Path,
        base: &Path,
    ) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read partials dir: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::register_partials_recursive(hbs, &path, base)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("hbs") {
                let content = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read partial: {}", path.display()))?;

                let rel = path.strip_prefix(base).unwrap_or(&path);
                let name = rel.with_extension("").to_string_lossy().replace('\\', "/");

                hbs.register_partial(&name, &content)
                    .with_context(|| format!("Failed to register partial: {name}"))?;
            }
        }
        Ok(())
    }

    fn register_templates(hbs: &mut Handlebars<'static>, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read templates dir: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("hbs") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                hbs.register_template_file(name, &path)
                    .with_context(|| format!("Failed to register template: {}", path.display()))?;
            }
        }
        Ok(())
    }

    pub fn render(&self, template: &str, data: &Value) -> Result<String> {
        self.hbs
            .render(template, data)
            .with_context(|| format!("Failed to render template: {template}"))
    }
}

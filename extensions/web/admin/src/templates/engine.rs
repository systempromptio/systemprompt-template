use std::path::Path;
use std::sync::Arc;

use handlebars::Handlebars;
use serde_json::Value;
use thiserror::Error;

use super::helpers;
use systemprompt_web_shared::BrandingConfig;

/// Errors raised while building or rendering the admin Handlebars engine.
#[derive(Debug, Error)]
pub enum AdminTemplateError {
    #[error("failed to read {context}: {source}")]
    ReadDir {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to read template file {path}: {source}")]
    ReadFile {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to register partial '{name}': {source}")]
    RegisterPartial {
        name: String,
        #[source]
        source: Box<handlebars::TemplateError>,
    },

    #[error("failed to register template '{name}': {source}")]
    RegisterTemplate {
        name: String,
        #[source]
        source: Box<handlebars::TemplateError>,
    },

    #[error("failed to render template '{template}': {source}")]
    Render {
        template: String,
        #[source]
        source: Box<handlebars::RenderError>,
    },
}

type Result<T> = std::result::Result<T, AdminTemplateError>;

#[derive(Clone, Debug)]
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
    pub const fn branding(&self) -> Option<&BrandingConfig> {
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
        let entries = std::fs::read_dir(dir).map_err(|source| AdminTemplateError::ReadDir {
            context: format!("partials dir: {}", dir.display()),
            source,
        })?;

        for entry in entries {
            let entry = entry.map_err(|source| AdminTemplateError::ReadDir {
                context: format!("partials dir entry: {}", dir.display()),
                source,
            })?;
            let path = entry.path();
            if path.is_dir() {
                Self::register_partials_recursive(hbs, &path, base)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("hbs") {
                let content = std::fs::read_to_string(&path).map_err(|source| {
                    AdminTemplateError::ReadFile {
                        path: path.display().to_string(),
                        source,
                    }
                })?;

                let rel = path.strip_prefix(base).unwrap_or(&path);
                let name = rel.with_extension("").to_string_lossy().replace('\\', "/");

                hbs.register_partial(&name, &content).map_err(|source| {
                    AdminTemplateError::RegisterPartial {
                        name: name.clone(),
                        source: Box::new(source),
                    }
                })?;
            }
        }
        Ok(())
    }

    fn register_templates(hbs: &mut Handlebars<'static>, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        let entries = std::fs::read_dir(dir).map_err(|source| AdminTemplateError::ReadDir {
            context: format!("templates dir: {}", dir.display()),
            source,
        })?;

        for entry in entries {
            let entry = entry.map_err(|source| AdminTemplateError::ReadDir {
                context: format!("templates dir entry: {}", dir.display()),
                source,
            })?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("hbs") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                hbs.register_template_file(name, &path).map_err(|source| {
                    AdminTemplateError::RegisterTemplate {
                        name: name.to_owned(),
                        source: Box::new(source),
                    }
                })?;
            }
        }
        Ok(())
    }

    pub fn render(&self, template: &str, data: &Value) -> Result<String> {
        self.hbs
            .render(template, data)
            .map_err(|source| AdminTemplateError::Render {
                template: template.to_owned(),
                source: Box::new(source),
            })
    }
}

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaperSection {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub image_alt: Option<String>,
    #[serde(default = "default_image_position")]
    pub image_position: String,
}

fn default_image_position() -> String {
    "right".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaperMetadata {
    #[serde(default)]
    pub hero_image: Option<String>,
    #[serde(default)]
    pub hero_alt: Option<String>,
    #[serde(default)]
    pub sections: Vec<PaperSection>,
    #[serde(default)]
    pub toc: bool,
    #[serde(default)]
    pub chapters_path: Option<String>,
}

impl PaperMetadata {
    pub fn validate(&self) -> Result<()> {
        if self.sections.is_empty() {
            return Err(anyhow!("Paper must have at least one section"));
        }

        for section in &self.sections {
            if section.id.is_empty() {
                return Err(anyhow!("Section id cannot be empty"));
            }
            if section.title.is_empty() {
                return Err(anyhow!("Section '{}' must have a title", section.id));
            }
        }

        let has_file_refs = self.sections.iter().any(|s| s.file.is_some());

        if has_file_refs {
            let chapters_path = self.chapters_path.as_ref().ok_or_else(|| {
                anyhow!("chapters_path is required when sections reference files")
            })?;

            if chapters_path.is_empty() {
                return Err(anyhow!("chapters_path cannot be empty"));
            }

            let chapters_dir = Path::new(chapters_path);
            if !chapters_dir.exists() {
                return Err(anyhow!("chapters_path '{}' does not exist", chapters_path));
            }
            if !chapters_dir.is_dir() {
                return Err(anyhow!(
                    "chapters_path '{}' is not a directory",
                    chapters_path
                ));
            }

            for section in &self.sections {
                if let Some(file) = &section.file {
                    let file_path = chapters_dir.join(file);
                    if !file_path.exists() {
                        return Err(anyhow!(
                            "Section '{}' references file '{}' which does not exist at '{}'",
                            section.id,
                            file,
                            file_path.display()
                        ));
                    }
                    if !file_path.is_file() {
                        return Err(anyhow!(
                            "Section '{}' references '{}' which is not a file",
                            section.id,
                            file
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn validate_section_ids_unique(&self) -> Result<()> {
        let mut seen_ids = std::collections::HashSet::new();
        for section in &self.sections {
            if !seen_ids.insert(&section.id) {
                return Err(anyhow!("Duplicate section id: '{}'", section.id));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedSection {
    pub id: String,
    pub title: String,
    pub content: String,
    pub image: Option<String>,
    pub image_alt: Option<String>,
    pub image_position: String,
}

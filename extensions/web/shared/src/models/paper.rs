use serde::{Deserialize, Serialize};

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

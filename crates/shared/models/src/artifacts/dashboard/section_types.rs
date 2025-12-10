use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SectionType {
    MetricsCards,
    Table,
    Chart,
    Timeline,
    Status,
    List,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct SectionLayout {
    pub width: LayoutWidth,
    pub order: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LayoutWidth {
    Full,
    Half,
    Third,
    TwoThirds,
}

impl Default for SectionLayout {
    fn default() -> Self {
        Self {
            width: LayoutWidth::Full,
            order: 0,
        }
    }
}

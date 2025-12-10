use super::section_types::{SectionLayout, SectionType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DashboardSection {
    pub section_id: String,
    pub title: String,
    pub section_type: SectionType,
    pub data: JsonValue,
    pub layout: SectionLayout,
}

impl DashboardSection {
    pub fn new(
        section_id: impl Into<String>,
        title: impl Into<String>,
        section_type: SectionType,
    ) -> Self {
        Self {
            section_id: section_id.into(),
            title: title.into(),
            section_type,
            data: JsonValue::Object(serde_json::Map::new()),
            layout: SectionLayout::default(),
        }
    }

    #[allow(clippy::expect_used)]
    pub fn with_data<T: Serialize>(mut self, data: T) -> Self {
        self.data = serde_json::to_value(data).expect("Failed to serialize section data");
        self
    }

    pub const fn with_layout(mut self, layout: SectionLayout) -> Self {
        self.layout = layout;
        self
    }

    pub const fn with_order(mut self, order: u32) -> Self {
        self.layout.order = order;
        self
    }
}

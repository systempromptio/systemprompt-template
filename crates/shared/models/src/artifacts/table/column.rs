use crate::artifacts::types::{Alignment, ColumnType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Column {
    pub name: String,
    #[serde(rename = "column_type")]
    pub kind: ColumnType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<Alignment>,
}

impl Column {
    pub fn new(name: impl Into<String>, kind: ColumnType) -> Self {
        Self {
            name: name.into(),
            kind,
            label: None,
            width: None,
            align: None,
        }
    }

    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.label = Some(header.into());
        self
    }

    pub fn with_label(self, label: impl Into<String>) -> Self {
        self.with_header(label)
    }

    pub const fn with_width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    pub const fn with_alignment(mut self, align: Alignment) -> Self {
        self.align = Some(align);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn column_type(&self) -> ColumnType {
        self.kind
    }
}

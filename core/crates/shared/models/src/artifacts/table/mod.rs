pub mod column;
pub mod hints;

pub use column::Column;
pub use hints::TableHints;

use crate::artifacts::metadata::ExecutionMetadata;
use crate::artifacts::traits::Artifact;
use crate::artifacts::types::ArtifactType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TableResponse {
    #[serde(rename = "x-artifact-type")]
    pub artifact_type: String,
    pub columns: Vec<Column>,
    pub items: Vec<JsonValue>,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<JsonValue>")]
    pub hints: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TableArtifact {
    #[serde(rename = "x-artifact-type")]
    #[serde(default = "default_artifact_type")]
    pub artifact_type: String,
    pub columns: Vec<Column>,
    pub items: Vec<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<JsonValue>")]
    pub hints: Option<JsonValue>,
    #[serde(skip)]
    #[schemars(skip)]
    hints_builder: TableHints,
    #[serde(skip)]
    #[schemars(skip)]
    metadata: ExecutionMetadata,
}

fn default_artifact_type() -> String {
    "table".to_string()
}

impl TableArtifact {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            artifact_type: "table".to_string(),
            columns,
            items: Vec::new(),
            hints: None,
            hints_builder: TableHints::default(),
            metadata: ExecutionMetadata::default(),
        }
    }

    pub fn with_rows(mut self, items: Vec<JsonValue>) -> Self {
        self.items = items;
        self
    }

    pub fn with_hints(mut self, hints: TableHints) -> Self {
        use crate::artifacts::traits::ArtifactSchema;
        self.hints = Some(hints.generate_schema());
        self.hints_builder = hints;
        self
    }

    pub fn with_metadata(mut self, metadata: ExecutionMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_execution_id(mut self, id: String) -> Self {
        self.metadata.execution_id = Some(id);
        self
    }

    pub fn with_skill(
        mut self,
        skill_id: impl Into<String>,
        skill_name: impl Into<String>,
    ) -> Self {
        self.metadata = self.metadata.with_skill(skill_id.into(), skill_name.into());
        self
    }

    pub fn to_response(&self) -> JsonValue {
        use crate::artifacts::traits::ArtifactSchema;

        let response = TableResponse {
            artifact_type: "table".to_string(),
            columns: self.columns.clone(),
            items: self.items.clone(),
            count: self.items.len(),
            execution_id: self.metadata.execution_id.clone(),
            hints: Some(self.hints_builder.generate_schema()),
        };
        serde_json::to_value(response).unwrap_or(JsonValue::Null)
    }
}

impl Artifact for TableArtifact {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::Table
    }

    fn to_schema(&self) -> JsonValue {
        use crate::artifacts::traits::ArtifactSchema;

        json!({
            "type": "object",
            "properties": {
                "columns": {
                    "type": "array",
                    "description": "Column definitions"
                },
                "items": {
                    "type": "array",
                    "description": "Array of data records"
                },
                "count": {
                    "type": "integer",
                    "description": "Total number of records"
                },
                "_execution_id": {
                    "type": "string",
                    "description": "Execution ID for tracking"
                }
            },
            "required": ["columns", "items"],
            "x-artifact-type": "table",
            "x-table-hints": self.hints_builder.generate_schema()
        })
    }
}

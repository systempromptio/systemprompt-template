use crate::artifacts::metadata::ExecutionMetadata;
use crate::artifacts::traits::Artifact;
use crate::artifacts::types::{ArtifactType, AxisType, ChartType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

fn default_artifact_type() -> String {
    "chart".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChartDataset {
    pub label: String,
    pub data: Vec<f64>,
}

impl ChartDataset {
    pub fn new(label: impl Into<String>, data: Vec<f64>) -> Self {
        Self {
            label: label.into(),
            data,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChartArtifact {
    #[serde(rename = "x-artifact-type")]
    #[serde(default = "default_artifact_type")]
    pub artifact_type: String,
    pub labels: Vec<String>,
    pub datasets: Vec<ChartDataset>,
    #[serde(skip)]
    #[schemars(skip)]
    chart_type: ChartType,
    #[serde(skip)]
    #[schemars(skip)]
    title: String,
    #[serde(skip)]
    #[schemars(skip)]
    x_axis_label: String,
    #[serde(skip)]
    #[schemars(skip)]
    y_axis_label: String,
    #[serde(skip)]
    #[schemars(skip)]
    x_axis_type: AxisType,
    #[serde(skip)]
    #[schemars(skip)]
    y_axis_type: AxisType,
    #[serde(skip)]
    #[schemars(skip)]
    metadata: ExecutionMetadata,
}

impl ChartArtifact {
    pub fn new(title: impl Into<String>, chart_type: ChartType) -> Self {
        Self {
            artifact_type: "chart".to_string(),
            labels: Vec::new(),
            datasets: Vec::new(),
            chart_type,
            title: title.into(),
            x_axis_label: "X".to_string(),
            y_axis_label: "Y".to_string(),
            x_axis_type: AxisType::Category,
            y_axis_type: AxisType::Linear,
            metadata: ExecutionMetadata::default(),
        }
    }

    pub fn with_x_axis_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    pub fn with_labels(self, labels: Vec<String>) -> Self {
        self.with_x_axis_labels(labels)
    }

    pub fn with_datasets(mut self, datasets: Vec<ChartDataset>) -> Self {
        self.datasets = datasets;
        self
    }

    pub fn add_dataset(mut self, dataset: ChartDataset) -> Self {
        self.datasets.push(dataset);
        self
    }

    pub const fn with_x_axis_type(mut self, axis_type: AxisType) -> Self {
        self.x_axis_type = axis_type;
        self
    }

    pub const fn with_y_axis_type(mut self, axis_type: AxisType) -> Self {
        self.y_axis_type = axis_type;
        self
    }

    pub fn with_axes(mut self, x_label: impl Into<String>, y_label: impl Into<String>) -> Self {
        self.x_axis_label = x_label.into();
        self.y_axis_label = y_label.into();
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
}

impl Artifact for ChartArtifact {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::Chart
    }

    fn to_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "labels": {
                    "type": "array",
                    "items": {"type": "string"}
                },
                "datasets": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "label": {"type": "string"},
                            "data": {"type": "array", "items": {"type": "number"}}
                        }
                    }
                },
                "_execution_id": {"type": "string"}
            },
            "required": ["labels", "datasets"],
            "x-artifact-type": "chart",
            "x-chart-hints": {
                "chart_type": self.chart_type,
                "title": self.title,
                "x_axis": {
                    "label": self.x_axis_label,
                    "type": self.x_axis_type
                },
                "y_axis": {
                    "label": self.y_axis_label,
                    "type": self.y_axis_type
                }
            }
        })
    }
}

//! A2A Artifact domain types
//!
//! Artifact and attachment entities for tasks.

use super::message::Part;
use serde::{Deserialize, Serialize};
use systemprompt_models::ArtifactMetadata;

/// Task artifact entity
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub artifact_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub parts: Vec<Part>,
    pub extensions: Vec<serde_json::Value>,
    pub metadata: ArtifactMetadata,
}

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt_identifiers::FileId;

use super::metadata::FileMetadata;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(clippy::struct_field_names)]
pub struct File {
    #[sqlx(rename = "id")]
    pub id: uuid::Uuid,
    pub file_path: String,
    pub public_url: String,
    pub mime_type: String,
    pub file_size_bytes: Option<i64>,
    pub ai_content: bool,
    pub metadata: serde_json::Value,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl File {
    pub fn id(&self) -> FileId {
        FileId::new(self.id.to_string())
    }

    pub fn metadata(&self) -> Result<FileMetadata> {
        serde_json::from_value(self.metadata.clone())
            .map_err(|e| anyhow!("Failed to deserialize metadata: {e}"))
    }
}

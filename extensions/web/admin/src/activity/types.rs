use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use super::enums::{ActivityAction, ActivityCategory, ActivityEntity};

#[derive(Debug)]
pub struct ActivityEntityRef {
    pub entity_type: ActivityEntity,
    pub entity_id: Option<String>,
    pub entity_name: Option<String>,
}

#[derive(Debug)]
pub struct NewActivity {
    pub user_id: String,
    pub category: ActivityCategory,
    pub action: ActivityAction,
    pub entity: Option<ActivityEntityRef>,
    pub description: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityTimelineEvent {
    pub id: String,
    pub user_id: String,
    pub display_name: String,
    pub category: ActivityCategory,
    pub action: ActivityAction,
    pub entity_type: Option<String>,
    pub entity_name: Option<String>,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityCategorySummary {
    pub category: String,
    pub count: i64,
}

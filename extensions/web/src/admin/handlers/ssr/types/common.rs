use crate::admin::repositories::user_plugins::AssociatedEntity;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct NamedEntity {
    pub id: String,
    pub name: String,
}

impl From<&AssociatedEntity> for NamedEntity {
    fn from(e: &AssociatedEntity) -> Self {
        Self {
            id: e.id.clone(),
            name: e.name.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckableEntity {
    pub value: String,
    pub name: String,
    pub checked: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventBreakdownView {
    pub event_type: String,
    pub event_count: i64,
    pub error_count: i64,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub pct: i64,
    pub avg_quality: String,
    pub quality_goal_pct: String,
    pub quality_sessions: i64,
}

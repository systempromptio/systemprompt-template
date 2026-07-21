use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::{SessionId, SkillId};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SessionEntityLink {
    pub entity_type: String,
    pub entity_name: String,
    pub usage_count: i32,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SessionRating {
    pub session_id: SessionId,
    pub rating: i16,
    pub outcome: String,
    pub notes: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SkillRating {
    pub skill_name: String,
    pub rating: i16,
    pub notes: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SkillEffectiveness {
    pub skill_name: String,
    pub skill_id: SkillId,
    pub total_uses: i64,
    pub sessions_used_in: i64,
    pub avg_effectiveness: f64,
    pub scored_sessions: i64,
    pub goal_achievement_pct: f64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EntityUsageSummary {
    pub entity_type: String,
    pub entity_name: String,
    // Why: polymorphic entity reference (skill/agent/mcp), no single typed-ID equivalent
    pub entity_id: String,
    pub total_uses: i64,
    pub session_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct RateSessionRequest {
    pub session_id: SessionId,
    pub rating: i16,
    #[serde(default)]
    pub outcome: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct RateSkillRequest {
    pub skill_name: String,
    pub rating: i16,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EntityEffectiveness {
    pub entity_name: String,
    pub total_uses: i64,
    pub sessions_used_in: i64,
    pub avg_effectiveness: f64,
    pub scored_sessions: i64,
    pub goal_achievement_pct: f64,
}

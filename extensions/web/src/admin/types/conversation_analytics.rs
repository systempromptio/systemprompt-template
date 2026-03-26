use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SessionEntityLink {
    pub entity_type: String,
    pub entity_name: String,
    pub usage_count: i32,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SessionRating {
    pub session_id: String,
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
    /// Canonical skill identifier (slug) for joining to `user_skills.skill_id`
    pub skill_id: String,
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
    /// Canonical identifier for joining (`skill_id` slug for skills, `entity_name` for others)
    pub entity_id: String,
    pub total_uses: i64,
    pub session_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct RateSessionRequest {
    pub session_id: String,
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

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EntityLastUsed {
    pub entity_type: String,
    pub entity_name: String,
    pub last_used: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EntityQualityTrend {
    pub entity_name: String,
    pub recent_avg: f64,
    pub previous_avg: f64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EntityHint {
    pub entity_name: String,
    pub hint: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct HookSessionQuality {
    pub event_type: String,
    pub session_count: i64,
    pub avg_quality: f64,
    pub goal_achievement_pct: f64,
}

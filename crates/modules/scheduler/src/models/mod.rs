use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use systemprompt_models::services::{JobConfig, SchedulerConfig};

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("Job not found: {job_name}")]
    JobNotFound { job_name: String },

    #[error("Invalid cron schedule: {schedule}")]
    InvalidSchedule { schedule: String },

    #[error("Job execution failed: {job_name} - {error}")]
    JobExecutionFailed { job_name: String, error: String },

    #[error("Database operation failed")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Scheduler already running")]
    AlreadyRunning,

    #[error("Scheduler not initialized")]
    NotInitialized,
}

impl SchedulerError {
    pub fn job_not_found(job_name: impl Into<String>) -> Self {
        Self::JobNotFound {
            job_name: job_name.into(),
        }
    }

    pub fn invalid_schedule(schedule: impl Into<String>) -> Self {
        Self::InvalidSchedule {
            schedule: schedule.into(),
        }
    }

    pub fn job_execution_failed(job_name: impl Into<String>, error: impl Into<String>) -> Self {
        Self::JobExecutionFailed {
            job_name: job_name.into(),
            error: error.into(),
        }
    }

    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScheduledJob {
    pub id: String,
    pub job_name: String,
    pub schedule: String,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub run_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiEvaluationResponse {
    pub agent_goal: String,
    pub goal_achieved: String,
    pub goal_achievement_confidence: f64,
    pub goal_achievement_notes: Option<String>,

    pub primary_category: String,
    pub topics_discussed: String,
    pub keywords: String,

    pub user_satisfied: i32,
    pub conversation_quality: i32,
    pub quality_notes: Option<String>,
    pub issues_encountered: Option<String>,

    pub completion_status: String,
    pub overall_score: f64,
    pub evaluation_summary: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ConversationEvaluation {
    pub id: Option<i32>,
    pub context_id: String,

    pub agent_goal: String,
    pub goal_achieved: String,
    pub goal_achievement_confidence: f32,
    pub goal_achievement_notes: Option<String>,

    pub primary_category: String,
    pub topics_discussed: String,
    pub keywords: String,

    pub user_satisfied: i32,
    pub conversation_quality: i32,
    pub quality_notes: Option<String>,
    pub issues_encountered: Option<String>,

    pub agent_name: String,
    pub total_turns: i32,
    pub conversation_duration_seconds: i32,
    pub user_initiated: bool,
    pub completion_status: String,

    pub overall_score: f32,
    pub evaluation_summary: String,
    pub analyzed_at: Option<DateTime<Utc>>,
    pub analysis_version: Option<String>,
}

impl ConversationEvaluation {
    pub fn from_ai_response(
        ai_response: AiEvaluationResponse,
        context_id: String,
        agent_name: String,
        total_turns: i32,
        conversation_duration_seconds: i32,
    ) -> Self {
        let normalized_category = normalize_category(&ai_response.primary_category);
        let normalized_status = normalize_completion_status(&ai_response.completion_status);
        let validated_score = validate_overall_score(ai_response.overall_score) as f32;

        Self {
            id: None,
            context_id,
            agent_name,
            total_turns,
            conversation_duration_seconds,
            user_initiated: true,
            analyzed_at: Some(Utc::now()),
            analysis_version: Some("v4".to_string()),
            agent_goal: ai_response.agent_goal,
            goal_achieved: ai_response.goal_achieved,
            goal_achievement_confidence: ai_response.goal_achievement_confidence as f32,
            goal_achievement_notes: ai_response.goal_achievement_notes,
            primary_category: normalized_category,
            topics_discussed: ai_response.topics_discussed,
            keywords: ai_response.keywords,
            user_satisfied: ai_response.user_satisfied,
            conversation_quality: ai_response.conversation_quality,
            quality_notes: ai_response.quality_notes,
            issues_encountered: ai_response.issues_encountered,
            completion_status: normalized_status,
            overall_score: validated_score,
            evaluation_summary: ai_response.evaluation_summary,
        }
    }
}

fn normalize_category(category: &str) -> String {
    match category.trim().to_lowercase().as_str() {
        "development" | "programming" | "coding" => "development".to_string(),
        "web development" | "web dev" | "webdev" => "web_development".to_string(),
        "system administration" | "sysadmin" | "sys admin" | "operations" => {
            "system_administration".to_string()
        },
        "content" | "content creation" | "writing" => "content_creation".to_string(),
        "configuration" | "config" => "configuration".to_string(),
        "information retrieval" | "research" => "information_retrieval".to_string(),
        "documentation" | "docs" => "documentation".to_string(),
        "language" | "linguistics" => "language".to_string(),
        "debugging" | "troubleshooting" => "debugging".to_string(),
        other => other.replace(" ", "_").to_lowercase(),
    }
}

fn normalize_completion_status(status: &str) -> String {
    match status.trim().to_lowercase().as_str() {
        "completed" | "complete" | "finished" | "success" | "successful" => "completed".to_string(),
        "abandoned" | "abandoned_by_user" | "skipped" | "cancelled" | "cancel" => {
            "abandoned".to_string()
        },
        "error" | "failed" | "failure" | "error_occurred" => "error".to_string(),
        _ => "completed".to_string(),
    }
}

fn validate_overall_score(score: f64) -> f64 {
    if score.is_nan() || score.is_infinite() {
        0.5
    } else if score < 0.0 {
        0.0
    } else if score > 1.0 {
        1.0
    } else {
        score
    }
}

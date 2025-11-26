use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub job_name: String,
    pub schedule: String,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
    pub last_error: Option<String>,
    pub run_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ScheduledJob {
    pub fn from_json_row(row: &systemprompt_core_database::JsonRow) -> anyhow::Result<Self> {
        use anyhow::anyhow;

        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let job_name = row
            .get("job_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing job_name"))?
            .to_string();

        let schedule = row
            .get("schedule")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing schedule"))?
            .to_string();

        let enabled = row
            .get("enabled")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow!("Missing enabled"))?;

        let last_run = row
            .get("last_run")
            .and_then(systemprompt_core_database::parse_database_datetime);

        let next_run = row
            .get("next_run")
            .and_then(systemprompt_core_database::parse_database_datetime);

        let last_status = row
            .get("last_status")
            .and_then(|v| v.as_str())
            .map(String::from);

        let last_error = row
            .get("last_error")
            .and_then(|v| v.as_str())
            .map(String::from);

        let run_count = row
            .get("run_count")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow!("Missing run_count"))?;

        let created_at = row
            .get("created_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid created_at"))?;

        let updated_at = row
            .get("updated_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid updated_at"))?;

        Ok(Self {
            id,
            job_name,
            schedule,
            enabled,
            last_run,
            next_run,
            last_status,
            last_error,
            run_count,
            created_at,
            updated_at,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JobConfig {
    pub name: String,
    pub enabled: bool,
    pub schedule: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub jobs: Vec<JobConfig>,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            jobs: vec![
                JobConfig {
                    name: "cleanup_anonymous_users".to_string(),
                    enabled: true,
                    schedule: "0 0 3 * * *".to_string(),
                },
                JobConfig {
                    name: "cleanup_inactive_sessions".to_string(),
                    enabled: true,
                    schedule: "0 0 * * * *".to_string(),
                },
                JobConfig {
                    name: "database_cleanup".to_string(),
                    enabled: true,
                    schedule: "0 0 4 * * *".to_string(),
                },
                JobConfig {
                    name: "ingest_content".to_string(),
                    enabled: true,
                    schedule: "0 */30 * * * *".to_string(), // Every 30 minutes
                },
                JobConfig {
                    name: "regenerate_static_content".to_string(),
                    enabled: true,
                    schedule: "0 0 0 * * *".to_string(), // Daily at midnight
                },
                JobConfig {
                    name: "evaluate_conversations".to_string(),
                    enabled: true,
                    schedule: "0 */30 * * * *".to_string(), // Every 30 minutes
                },
            ],
        }
    }
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

#[derive(Debug, Clone, Serialize)]
pub struct ConversationEvaluation {
    pub id: Option<i64>,
    pub context_id: String,

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

    pub agent_name: String,
    pub total_turns: i32,
    pub conversation_duration_seconds: i32,
    pub user_initiated: bool,
    pub completion_status: String,

    pub overall_score: f64,
    pub evaluation_summary: String,
    pub analyzed_at: Option<DateTime<Utc>>,
    pub analysis_version: String,
}

impl ConversationEvaluation {
    pub fn from_json_row(row: &systemprompt_core_database::JsonRow) -> anyhow::Result<Self> {
        use anyhow::anyhow;

        let id = row.get("id").and_then(|v| v.as_i64());

        let context_id = row
            .get("context_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing context_id"))?
            .to_string();

        let agent_goal = row
            .get("agent_goal")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: agent_goal"))?
            .to_string();

        let goal_achieved = row
            .get("goal_achieved")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: goal_achieved"))?
            .to_string();

        let goal_achievement_confidence = row
            .get("goal_achievement_confidence")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing required field: goal_achievement_confidence"))?;

        let primary_category = row
            .get("primary_category")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: primary_category"))?
            .to_string();

        let topics_discussed = row
            .get("topics_discussed")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: topics_discussed"))?
            .to_string();

        let keywords = row
            .get("keywords")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: keywords"))?
            .to_string();

        let user_satisfied = row
            .get("user_satisfied")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow!("Missing required field: user_satisfied"))?
            as i32;

        let conversation_quality = row
            .get("conversation_quality")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow!("Missing required field: conversation_quality"))?
            as i32;

        let agent_name = row
            .get("agent_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: agent_name"))?
            .to_string();

        let total_turns =
            row.get("total_turns")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow!("Missing required field: total_turns"))? as i32;

        let conversation_duration_seconds = row
            .get("conversation_duration_seconds")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| anyhow!("Missing required field: conversation_duration_seconds"))?
            as i32;

        let user_initiated = row
            .get("user_initiated")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow!("Missing required field: user_initiated"))?;

        let completion_status = row
            .get("completion_status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: completion_status"))?
            .to_string();

        let overall_score = row
            .get("overall_score")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| anyhow!("Missing required field: overall_score"))?;

        let evaluation_summary = row
            .get("evaluation_summary")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required field: evaluation_summary"))?
            .to_string();

        Ok(Self {
            id,
            context_id,
            agent_goal,
            goal_achieved,
            goal_achievement_confidence,
            goal_achievement_notes: row
                .get("goal_achievement_notes")
                .and_then(|v| v.as_str())
                .map(String::from),
            primary_category,
            topics_discussed,
            keywords,
            user_satisfied,
            conversation_quality,
            quality_notes: row
                .get("quality_notes")
                .and_then(|v| v.as_str())
                .map(String::from),
            issues_encountered: row
                .get("issues_encountered")
                .and_then(|v| v.as_str())
                .map(String::from),
            agent_name,
            total_turns,
            conversation_duration_seconds,
            user_initiated,
            completion_status,
            overall_score,
            evaluation_summary,
            analyzed_at: row
                .get("analyzed_at")
                .and_then(systemprompt_core_database::parse_database_datetime),
            analysis_version: row
                .get("analysis_version")
                .and_then(|v| v.as_str())
                .unwrap_or("v4")
                .to_string(),
        })
    }

    pub fn from_ai_response(
        ai_response: AiEvaluationResponse,
        context_id: String,
        agent_name: String,
        total_turns: i32,
        conversation_duration_seconds: i32,
    ) -> Self {
        let normalized_category = normalize_category(&ai_response.primary_category);
        let normalized_status = normalize_completion_status(&ai_response.completion_status);
        let validated_score = validate_overall_score(ai_response.overall_score);

        Self {
            id: None,
            context_id,
            agent_name,
            total_turns,
            conversation_duration_seconds,
            user_initiated: true,
            analyzed_at: Some(Utc::now()),
            analysis_version: "v4".to_string(),
            agent_goal: ai_response.agent_goal,
            goal_achieved: ai_response.goal_achieved,
            goal_achievement_confidence: ai_response.goal_achievement_confidence,
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

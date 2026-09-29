//! Plugin evaluation: one row per conversation that invoked a marketplace's
//! skill, scored by fixed rules over the conversation's own stored messages.
//!
//! The rules live in `plugin_eval.sql` and nowhere else: the Versions page's
//! Evaluation tab, the `analysis-plugin-eval` export and the eval harness all
//! read these rows, so the three can never disagree. A tool call is read from
//! every turn of the conversation, de-duplicated because each turn repeats
//! the thread before it, paired with the message after it as its result, and
//! classified by the error patterns in its result text. `plugin_eval_tools.sql`
//! shares those CTEs and groups the same calls by tool. No model scores
//! anything.
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, MarketplaceId, PluginId};

use crate::repositories::analysis::marketplace_versions::VersionWindow;

/// One conversation × skill under the marketplace version that served it.
#[derive(Debug, Clone, Serialize)]
pub struct PluginEvalRow {
    pub context_id: ContextId,
    pub client_session_id: Option<String>,
    pub plugin_id: PluginId,
    pub skill: String,
    pub marketplace_hash: String,
    pub first_invoked_at: DateTime<Utc>,
    pub turns: i64,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost: i64,
    pub duration_seconds: Option<i64>,
    pub p50_ms: Option<i32>,
    pub p95_ms: Option<i32>,
    pub mcp_calls: i64,
    pub connector_calls: i64,
    pub builtin_calls: i64,
    pub failed_calls: i64,
    pub schema_errors: i64,
    pub access_errors: i64,
    pub upstream_errors: i64,
    pub bad_arguments: i64,
    pub timeouts: i64,
    pub repeated_calls: i64,
    pub widgets: i64,
    pub writes: i64,
    pub answer_chars: i64,
    pub placeholder_mentions: i64,
    pub tools_unavailable: bool,
    pub completed: bool,
    pub success: bool,
}

/// One tool within one skill under one marketplace version: how often it was
/// called and how often it failed, by the same classes as a conversation row.
#[derive(Debug, Clone, Serialize)]
pub struct PluginEvalToolRow {
    pub marketplace_hash: String,
    pub plugin_id: PluginId,
    pub skill: String,
    pub tool: String,
    pub calls: i64,
    pub conversations: i64,
    pub failed_calls: i64,
    pub schema_errors: i64,
    pub access_errors: i64,
    pub upstream_errors: i64,
    pub bad_arguments: i64,
    pub timeouts: i64,
    pub repeated_calls: i64,
}

pub async fn list_plugin_eval_runs(
    pool: &PgPool,
    window: VersionWindow,
    marketplace_id: &MarketplaceId,
) -> Result<Vec<PluginEvalRow>, sqlx::Error> {
    sqlx::query_file_as!(
        PluginEvalRow,
        "src/repositories/analysis/plugin_eval.sql",
        window.start,
        window.end,
        marketplace_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn list_plugin_eval_tools(
    pool: &PgPool,
    window: VersionWindow,
    marketplace_id: &MarketplaceId,
) -> Result<Vec<PluginEvalToolRow>, sqlx::Error> {
    sqlx::query_file_as!(
        PluginEvalToolRow,
        "src/repositories/analysis/plugin_eval_tools.sql",
        window.start,
        window.end,
        marketplace_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

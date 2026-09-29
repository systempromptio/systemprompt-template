//! The harness hook plane of one conversation.
//!
//! Every event the bridge reported from inside the host for the client
//! session the conversation's requests carried, beside the gateway's own
//! record of the same turns.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::PluginId;

/// One harness hook event of the conversation's client session — the plane
/// the bridge reports from inside the host, beside the gateway's own record.
#[derive(Debug, Clone)]
pub struct ConversationHookEventRow {
    pub event_type: String,
    pub tool_name: Option<String>,
    pub plugin_id: Option<PluginId>,
    pub prompt_preview: Option<String>,
    pub description: Option<String>,
    // JSON: the hook payload as the harness posted it, shaped per event type.
    pub metadata: serde_json::Value,
    pub tool_use_id: Option<String>,
    pub trace_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_conversation_hook_events(
    pool: &PgPool,
    client_session_id: &str,
) -> Result<Vec<ConversationHookEventRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT e.event_type AS "event_type!", e.tool_name, e.plugin_id AS "plugin_id?: PluginId",
                  e.prompt_preview, e.description, COALESCE(e.metadata, '{}'::jsonb) AS "metadata!",
                  e.tool_use_id, e.trace_id, e.created_at AS "created_at!"
           FROM plugin_usage_events e
           WHERE e.session_id = $1
           ORDER BY e.created_at
           LIMIT 2000"#,
        client_session_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ConversationHookEventRow {
            event_type: r.event_type,
            tool_name: r.tool_name,
            plugin_id: r.plugin_id,
            prompt_preview: r.prompt_preview,
            description: r.description,
            metadata: r.metadata,
            tool_use_id: r.tool_use_id,
            trace_id: r.trace_id,
            created_at: r.created_at,
        })
        .collect())
}

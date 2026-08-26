//! Persisting captured conversation transcripts.
//!
//! One row per session, keyed `st-<session_id>` and overwritten on every
//! capture: the hook posts the full transcript each time, so the latest
//! payload supersedes prior ones and the history surface always reads a
//! complete conversation rather than fragments.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

// JSON: walks one transcript entry of the third-party Claude Code shape,
// where usage and model live either at the top level or under `message`.
fn entry_field<'a>(entry: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    entry
        .get(key)
        .or_else(|| entry.get("message").and_then(|m| m.get(key)))
}

fn usage_total(entry: &serde_json::Value, key: &str) -> i64 {
    entry_field(entry, "usage")
        .and_then(|u| u.get(key))
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0)
}

pub async fn upsert_session_transcript(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &SessionId,
    plugin_id: Option<&str>,
    transcript: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    let empty = Vec::new();
    let entries_vec = transcript.as_array().unwrap_or(&empty);
    let entries: i32 = i32::try_from(entries_vec.len()).unwrap_or(i32::MAX);
    let input_tokens: i64 = entries_vec
        .iter()
        .map(|e| usage_total(e, "input_tokens"))
        .sum();
    let output_tokens: i64 = entries_vec
        .iter()
        .map(|e| usage_total(e, "output_tokens"))
        .sum();
    // JSON: third-party Claude Code transcript shape; see entry_field above.
    let model = entries_vec
        .iter()
        .rev()
        .find_map(|e| entry_field(e, "model").and_then(serde_json::Value::as_str));
    sqlx::query!(
        r#"
        INSERT INTO session_transcripts
            (id, user_id, session_id, plugin_id, transcript,
             total_input_tokens, total_output_tokens, model, entries_counted, captured_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
        ON CONFLICT (id) DO UPDATE SET
            transcript = EXCLUDED.transcript,
            plugin_id = COALESCE(EXCLUDED.plugin_id, session_transcripts.plugin_id),
            total_input_tokens = EXCLUDED.total_input_tokens,
            total_output_tokens = EXCLUDED.total_output_tokens,
            model = COALESCE(EXCLUDED.model, session_transcripts.model),
            entries_counted = EXCLUDED.entries_counted,
            captured_at = NOW()
        "#,
        format!("st-{}", session_id.as_str()),
        user_id.as_str(),
        session_id.as_str(),
        plugin_id,
        transcript,
        input_tokens,
        output_tokens,
        model,
        entries,
    )
    .execute(pool)
    .await?;
    Ok(())
}

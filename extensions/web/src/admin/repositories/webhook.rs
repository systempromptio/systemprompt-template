use std::sync::Arc;

use sqlx::PgPool;

pub async fn insert_plugin_usage_event(
    pool: &Arc<PgPool>,
    user_id: &str,
    session_id: &str,
    event_type: &str,
    tool_name: Option<&str>,
    plugin_id: Option<&str>,
    metadata: &serde_json::Value,
) -> Result<bool, anyhow::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now();
    let ts_second = now.format("%Y-%m-%dT%H:%M:%S").to_string();
    let dedup_key = format!(
        "{}|{}|{}|{}|{}",
        user_id,
        session_id,
        event_type,
        tool_name.unwrap_or(""),
        ts_second,
    );

    let result = sqlx::query(
        "INSERT INTO plugin_usage_events (id, user_id, session_id, event_type, tool_name, plugin_id, metadata, dedup_key)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (dedup_key) WHERE dedup_key IS NOT NULL DO NOTHING",
    )
    .bind(&id)
    .bind(user_id)
    .bind(session_id)
    .bind(event_type)
    .bind(tool_name)
    .bind(plugin_id)
    .bind(metadata)
    .bind(&dedup_key)
    .execute(pool.as_ref())
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn insert_session_transcript(
    pool: &Arc<PgPool>,
    user_id: &str,
    session_id: &str,
    plugin_id: Option<&str>,
    transcript: &serde_json::Value,
) -> Result<String, anyhow::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO session_transcripts (id, user_id, session_id, plugin_id, transcript)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&id)
    .bind(user_id)
    .bind(session_id)
    .bind(plugin_id)
    .bind(transcript)
    .execute(pool.as_ref())
    .await?;

    Ok(id)
}

pub async fn get_session_entries_counted(
    pool: &PgPool,
    session_id: &str,
) -> Result<i32, anyhow::Error> {
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT COALESCE(MAX(entries_counted), 0) FROM session_transcripts WHERE session_id = $1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map_or(0, |r| r.0))
}

pub async fn update_transcript_tokens(
    pool: &PgPool,
    transcript_id: &str,
    input_tokens: i64,
    output_tokens: i64,
    model: Option<&str>,
    entries_counted: i32,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        "UPDATE session_transcripts
         SET total_input_tokens = $2, total_output_tokens = $3, model = $4, entries_counted = $5
         WHERE id = $1",
    )
    .bind(transcript_id)
    .bind(input_tokens)
    .bind(output_tokens)
    .bind(model)
    .bind(entries_counted)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn upsert_plugin_installation(
    pool: &PgPool,
    user_id: &str,
    plugin_id: &str,
    plugin_version: &str,
    plugin_source: &str,
    base_plugin_id: Option<&str>,
) -> Result<String, anyhow::Error> {
    let existing: Option<(String, String)> = sqlx::query_as(
        "SELECT id, plugin_version FROM plugin_installations WHERE user_id = $1 AND plugin_id = $2",
    )
    .bind(user_id)
    .bind(plugin_id)
    .fetch_optional(pool)
    .await?;

    if let Some((existing_id, old_version)) = existing {
        sqlx::query(
            "UPDATE plugin_installations
             SET last_seen_at = NOW(), session_count = session_count + 1, plugin_version = $3,
                 plugin_source = $4, base_plugin_id = $5
             WHERE id = $1",
        )
        .bind(&existing_id)
        .bind(user_id)
        .bind(plugin_version)
        .bind(plugin_source)
        .bind(base_plugin_id)
        .execute(pool)
        .await?;

        if old_version == plugin_version {
            Ok("seen".to_string())
        } else {
            let history_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO plugin_installation_history (id, user_id, plugin_id, plugin_version, event_type, previous_version, plugin_source, base_plugin_id)
                 VALUES ($1, $2, $3, $4, 'updated', $5, $6, $7)",
            )
            .bind(&history_id)
            .bind(user_id)
            .bind(plugin_id)
            .bind(plugin_version)
            .bind(&old_version)
            .bind(plugin_source)
            .bind(base_plugin_id)
            .execute(pool)
            .await?;

            Ok("updated".to_string())
        }
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO plugin_installations (id, user_id, plugin_id, plugin_version, plugin_source, base_plugin_id)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&id)
        .bind(user_id)
        .bind(plugin_id)
        .bind(plugin_version)
        .bind(plugin_source)
        .bind(base_plugin_id)
        .execute(pool)
        .await?;

        let history_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO plugin_installation_history (id, user_id, plugin_id, plugin_version, event_type, plugin_source, base_plugin_id)
             VALUES ($1, $2, $3, $4, 'installed', $5, $6)",
        )
        .bind(&history_id)
        .bind(user_id)
        .bind(plugin_id)
        .bind(plugin_version)
        .bind(plugin_source)
        .bind(base_plugin_id)
        .execute(pool)
        .await?;

        Ok("installed".to_string())
    }
}

pub struct TranscriptTokens {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub model: Option<String>,
    pub entries_processed: i32,
}

#[must_use]
pub fn extract_transcript_tokens(
    transcript: &serde_json::Value,
    skip_count: i32,
) -> TranscriptTokens {
    let Some(entries) = transcript.as_array() else {
        return TranscriptTokens {
            input_tokens: 0,
            output_tokens: 0,
            model: None,
            entries_processed: skip_count,
        };
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let total_entries = entries.len() as i32;
    let mut input_tokens: i64 = 0;
    let mut output_tokens: i64 = 0;
    let mut model: Option<String> = None;

    #[allow(clippy::cast_sign_loss)]
    for entry in entries.iter().skip(skip_count as usize) {
        if entry
            .get("isSidechain")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        {
            continue;
        }
        if entry
            .get("isApiErrorMessage")
            .and_then(serde_json::Value::as_bool)
            == Some(true)
        {
            continue;
        }

        let Some(message) = entry.get("message") else {
            continue;
        };

        if let Some(m) = message.get("model").and_then(|v| v.as_str()) {
            model = Some(m.to_string());
        }

        if let Some(usage) = message.get("usage") {
            if let Some(inp) = usage
                .get("input_tokens")
                .and_then(serde_json::Value::as_i64)
            {
                input_tokens += inp;
            }
            if let Some(out) = usage
                .get("output_tokens")
                .and_then(serde_json::Value::as_i64)
            {
                output_tokens += out;
            }
        }
    }

    TranscriptTokens {
        input_tokens,
        output_tokens,
        model,
        entries_processed: total_entries,
    }
}

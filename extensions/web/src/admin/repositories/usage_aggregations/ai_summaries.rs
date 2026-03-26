use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

struct AnalysisRow {
    session_id: String,
    summary: String,
    tags: String,
    title: String,
}

struct FallbackRow {
    session_id: String,
    ai_summary: Option<String>,
    ai_tags: Option<String>,
    ai_title: Option<String>,
}

pub async fn fetch_session_ai_summaries(
    pool: &PgPool,
    session_ids: &[String],
) -> Result<Vec<(String, String, String, String)>, sqlx::Error> {
    if session_ids.is_empty() {
        return Ok(Vec::new());
    }

    let analysis_rows = sqlx::query_as!(
        AnalysisRow,
        r"SELECT session_id, summary, tags, title
          FROM session_analyses
          WHERE session_id = ANY($1)",
        session_ids,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut result: Vec<(String, String, String, String)> = analysis_rows
        .into_iter()
        .map(|r| (r.session_id, r.summary, r.tags, r.title))
        .collect();
    let found_ids: std::collections::HashSet<&str> =
        result.iter().map(|(sid, _, _, _)| sid.as_str()).collect();

    let missing: Vec<&String> = session_ids
        .iter()
        .filter(|sid| !found_ids.contains(sid.as_str()))
        .collect();

    if !missing.is_empty() {
        let missing_vec: Vec<String> = missing.into_iter().cloned().collect();

        let fallback_rows = sqlx::query_as!(
            FallbackRow,
            r"SELECT session_id, ai_summary, ai_tags, ai_title
              FROM plugin_session_summaries
              WHERE session_id = ANY($1) AND ai_summary IS NOT NULL",
            &missing_vec,
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default();

        for r in fallback_rows {
            result.push((
                r.session_id,
                r.ai_summary.unwrap_or_default(),
                r.ai_tags.unwrap_or_default(),
                r.ai_title.unwrap_or_default(),
            ));
        }
    }

    Ok(result)
}

pub async fn update_session_ai_summary(
    pool: &PgPool,
    session_id: &SessionId,
    summary: &str,
    tags: &str,
) {
    update_session_ai_summary_with_title(pool, session_id, None, summary, tags).await;
}

pub async fn update_session_ai_summary_with_title(
    pool: &PgPool,
    session_id: &SessionId,
    title: Option<&str>,
    summary: &str,
    tags: &str,
) {
    let result = sqlx::query!(
        r"UPDATE plugin_session_summaries
          SET ai_summary = $2, ai_tags = $3, ai_title = COALESCE($4, ai_title), updated_at = NOW()
          WHERE session_id = $1",
        session_id.as_str(),
        summary,
        tags,
        title,
    )
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to update session AI summary");
    }
}

pub async fn update_session_ai_summary_structured(
    pool: &PgPool,
    session_id: &SessionId,
    summary: &super::super::super::handlers::hooks_track::ai_summary::SessionAnalysis,
) {
    let tags = summary.tags.join(",");
    let composed_summary = summary.composed_summary();
    let result = sqlx::query!(
        r"UPDATE plugin_session_summaries
          SET ai_title = $2, ai_description = $3, ai_summary = $4, ai_tags = $5, updated_at = NOW()
          WHERE session_id = $1",
        session_id.as_str(),
        summary.title,
        summary.description,
        composed_summary,
        tags,
    )
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to update structured session AI summary");
    }
}

//! Prefix matches for the global header search bar.
//!
//! Every list page renders ids through `short_id` (twelve characters and an
//! ellipsis), so what an operator copies is a prefix, never the whole id.
//! [`list_id_matches`] finds the entities whose id starts with that prefix
//! across the same tables [`super::resolve`] checks exactly, newest first, so
//! the box can offer them as completions and jump straight to a unique hit.

use sqlx::PgPool;

use super::resolve::{ResolvedId, ResolvedKind};

// Why: short of this a prefix is a single hex character matching every row in
// the table; the box asks for nothing shorter.
pub const MIN_PREFIX_LEN: usize = 4;

pub async fn list_id_matches(
    pool: &PgPool,
    prefix: &str,
    limit: i64,
) -> Result<Vec<ResolvedId>, sqlx::Error> {
    if prefix.chars().count() < MIN_PREFIX_LEN {
        return Ok(Vec::new());
    }
    let pattern = format!("{}%", escape_like(prefix));
    let rows = sqlx::query!(
        r#"SELECT kind AS "kind!", id AS "id!"
           FROM (
               SELECT 'request' AS kind, id, created_at FROM ai_requests WHERE id LIKE $1
               UNION ALL
               SELECT 'request', id, created_at FROM ai_requests WHERE request_id LIKE $1
               UNION ALL
               SELECT 'trace', trace_id, MAX(created_at) FROM ai_requests
               WHERE trace_id LIKE $1 GROUP BY trace_id
               UNION ALL
               SELECT 'trace', trace_id, MAX(created_at) FROM governance_decisions
               WHERE trace_id LIKE $1 GROUP BY trace_id
               UNION ALL
               SELECT 'session', session_id, MAX(created_at) FROM ai_requests
               WHERE session_id LIKE $1 GROUP BY session_id
               UNION ALL
               SELECT 'session', session_id, MAX(created_at) FROM governance_decisions
               WHERE session_id LIKE $1 GROUP BY session_id
               UNION ALL
               SELECT 'session', session_id, MAX(created_at) FROM user_contexts
               WHERE session_id LIKE $1 GROUP BY session_id
               UNION ALL
               SELECT 'context', context_id, MAX(created_at) FROM ai_requests
               WHERE context_id LIKE $1 GROUP BY context_id
               UNION ALL
               SELECT 'context', context_id, MAX(created_at) FROM user_contexts
               WHERE context_id LIKE $1 GROUP BY context_id
           ) t
           WHERE id IS NOT NULL
           ORDER BY created_at DESC
           LIMIT $2"#,
        pattern,
        limit,
    )
    .fetch_all(pool)
    .await?;

    let mut seen = std::collections::HashSet::new();
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            let kind = match r.kind.as_str() {
                "request" => ResolvedKind::Request,
                "trace" => ResolvedKind::Trace,
                "session" => ResolvedKind::Session,
                "context" => ResolvedKind::Context,
                _ => return None,
            };
            seen.insert((r.kind, r.id.clone()))
                .then_some(ResolvedId { kind, id: r.id })
        })
        .collect())
}

// Why: `_` and `%` are LIKE wildcards, and ids legitimately contain `_`
// (`sess_…`); a raw prefix would match far more than it reads.
fn escape_like(raw: &str) -> String {
    raw.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

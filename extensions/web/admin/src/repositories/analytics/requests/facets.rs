//! The values the request log's filter selects offer.
//!
//! One statement, not five: the options are read for the same window and
//! visibility floor as the rows, and issuing them together keeps the picker
//! provably consistent with the table under it. Each facet is capped, so a
//! long tail of one-off model names cannot turn a select into a scroll.

use sqlx::PgPool;

use crate::repositories::scope::SubjectScope;
use crate::util::time_range::TimeRange;

/// One selectable value and how many rows in the window carry it.
#[derive(Debug, Clone)]
pub struct FacetValue {
    pub kind: String,
    pub value: String,
    pub count: i64,
}

pub async fn list_request_facets(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<Vec<FacetValue>, sqlx::Error> {
    sqlx::query_as!(
        FacetValue,
        r#"WITH windowed AS (
            SELECT ar.id, ar.model, ar.provider, ar.status
            FROM ai_requests ar
            WHERE ar.created_at >= $1 AND ar.created_at < $2
              AND ($3::TEXT[] IS NULL OR ar.user_id = ANY($3))
        ),
        facets AS (
            (SELECT 'model' AS kind, model AS value, COUNT(*) AS n
               FROM windowed WHERE model IS NOT NULL
               GROUP BY model ORDER BY COUNT(*) DESC LIMIT 60)
            UNION ALL
            (SELECT 'provider', provider, COUNT(*)
               FROM windowed WHERE provider IS NOT NULL
               GROUP BY provider ORDER BY COUNT(*) DESC LIMIT 30)
            UNION ALL
            (SELECT 'status', status, COUNT(*)
               FROM windowed GROUP BY status ORDER BY COUNT(*) DESC LIMIT 30)
            UNION ALL
            (SELECT 'tool', tc.tool_name, COUNT(*)
               FROM ai_request_tool_calls tc
               JOIN windowed w ON w.id = tc.request_id
               GROUP BY tc.tool_name ORDER BY COUNT(*) DESC LIMIT 60)
        )
        SELECT kind AS "kind!", value AS "value!", n::bigint AS "count!"
        FROM facets
        ORDER BY kind ASC, n DESC, value ASC"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await
}

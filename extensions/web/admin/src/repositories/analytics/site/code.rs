//! Code-impact reads for the dashboard's code tab.
//!
//! Two measurement frames that the page never mixes: hook-observed AI line
//! deltas (`loc_added_ai` — lines Claude applied through Edit/Write) and git
//! commit diff totals (`commit_insertions` — AI and manual lines together,
//! only for commits made inside tracked sessions). Reads go to the
//! `admin_usage_daily_rollups` table the `usage_daily_rollup` job maintains.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Clone, Copy)]
pub struct CodeDayBucket {
    pub date: chrono::NaiveDate,
    pub commits: i64,
    pub commit_insertions: i64,
    pub commit_deletions: i64,
    pub loc_added_ai: i64,
    pub loc_removed_ai: i64,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CodeTotals {
    pub loc_added_ai: i64,
    pub loc_removed_ai: i64,
    pub commits: i64,
    pub commit_insertions: i64,
    pub commit_deletions: i64,
    // Why: Applied Edit/Write tool calls — the honest replacement for "accepts":
    // Claude Code emits no accept/reject signal, so this counts edits that
    // landed, with no denominator to rate them against.
    pub ai_edit_operations: i64,
}

pub async fn list_daily_code_series(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<Vec<CodeDayBucket>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        WITH spine AS (
            SELECT generate_series(
                DATE_TRUNC('day', $1::TIMESTAMPTZ),
                DATE_TRUNC('day', $2::TIMESTAMPTZ),
                INTERVAL '1 day'
            )::DATE AS date
        ),
        agg AS (
            SELECT
                ru.date,
                COALESCE(SUM(ru.commits_count), 0)::BIGINT AS commits,
                COALESCE(SUM(ru.commit_insertions), 0)::BIGINT AS insertions,
                COALESCE(SUM(ru.commit_deletions), 0)::BIGINT AS deletions,
                COALESCE(SUM(ru.loc_added_ai), 0)::BIGINT AS loc_added,
                COALESCE(SUM(ru.loc_removed_ai), 0)::BIGINT AS loc_removed
            FROM admin_usage_daily_rollups ru
            LEFT JOIN organization_members m ON m.user_id = ru.user_id
            LEFT JOIN organizations o ON o.id = m.org_id
            LEFT JOIN user_profile_ext upe ON upe.user_id = ru.user_id
            WHERE ru.date >= ($1::TIMESTAMPTZ)::DATE AND ru.date <= ($2::TIMESTAMPTZ)::DATE
              AND ($3::TEXT IS NULL OR o.slug = $3)
              AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
              AND ($5::TEXT IS NULL OR ru.user_id = $5)
            GROUP BY ru.date
        )
        SELECT
            s.date AS "date!",
            COALESCE(a.commits, 0)::BIGINT AS "commits!",
            COALESCE(a.insertions, 0)::BIGINT AS "insertions!",
            COALESCE(a.deletions, 0)::BIGINT AS "deletions!",
            COALESCE(a.loc_added, 0)::BIGINT AS "loc_added!",
            COALESCE(a.loc_removed, 0)::BIGINT AS "loc_removed!"
        FROM spine s
        LEFT JOIN agg a ON a.date = s.date
        ORDER BY s.date
        "#,
        range.from,
        range.to,
        scope.org_slug.as_slug(),
        scope.department.as_deref(),
        scope.user_id_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CodeDayBucket {
            date: r.date,
            commits: r.commits,
            commit_insertions: r.insertions,
            commit_deletions: r.deletions,
            loc_added_ai: r.loc_added,
            loc_removed_ai: r.loc_removed,
        })
        .collect())
}

pub async fn get_code_totals(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<CodeTotals, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COALESCE(SUM(ru.loc_added_ai), 0)::BIGINT AS "loc_added!",
            COALESCE(SUM(ru.loc_removed_ai), 0)::BIGINT AS "loc_removed!",
            COALESCE(SUM(ru.commits_count), 0)::BIGINT AS "commits!",
            COALESCE(SUM(ru.commit_insertions), 0)::BIGINT AS "insertions!",
            COALESCE(SUM(ru.commit_deletions), 0)::BIGINT AS "deletions!"
        FROM admin_usage_daily_rollups ru
        LEFT JOIN organization_members m ON m.user_id = ru.user_id
        LEFT JOIN organizations o ON o.id = m.org_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = ru.user_id
        WHERE ru.date >= ($1::TIMESTAMPTZ)::DATE AND ru.date <= ($2::TIMESTAMPTZ)::DATE
          AND ($3::TEXT IS NULL OR o.slug = $3)
          AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
          AND ($5::TEXT IS NULL OR ru.user_id = $5)
        "#,
        range.from,
        range.to,
        scope.org_slug.as_slug(),
        scope.department.as_deref(),
        scope.user_id_str(),
    )
    .fetch_one(pool)
    .await?;

    let edits = get_ai_edit_operations(pool, range, scope).await?;

    Ok(CodeTotals {
        loc_added_ai: row.loc_added,
        loc_removed_ai: row.loc_removed,
        commits: row.commits,
        commit_insertions: row.insertions,
        commit_deletions: row.deletions,
        ai_edit_operations: edits,
    })
}

async fn get_ai_edit_operations(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COALESCE(SUM(d.event_count), 0)::BIGINT AS "count!"
        FROM plugin_usage_daily d
        LEFT JOIN organization_members m ON m.user_id = d.user_id
        LEFT JOIN organizations o ON o.id = m.org_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = d.user_id
        WHERE d.date >= ($1::TIMESTAMPTZ)::DATE AND d.date <= ($2::TIMESTAMPTZ)::DATE
          AND d.event_type = 'PostToolUse'
          AND d.tool_name IN ('Edit', 'Write', 'MultiEdit', 'NotebookEdit')
          AND ($3::TEXT IS NULL OR o.slug = $3)
          AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
          AND ($5::TEXT IS NULL OR d.user_id = $5)
        "#,
        range.from,
        range.to,
        scope.org_slug.as_slug(),
        scope.department.as_deref(),
        scope.user_id_str(),
    )
    .fetch_one(pool)
    .await
}

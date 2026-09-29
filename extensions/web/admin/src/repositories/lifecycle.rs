//! The retention ledger the lifecycle jobs write: archives on disk, the
//! latest measurement per managed table, and the last health report.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct ArchiveRow {
    pub tier: String,
    pub period: String,
    pub table_name: String,
    pub relative_path: String,
    pub row_count: i64,
    pub byte_count: i64,
    pub sha256: String,
    pub window_from: DateTime<Utc>,
    pub window_to: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub async fn list_retention_archives(pool: &PgPool) -> Result<Vec<ArchiveRow>, sqlx::Error> {
    sqlx::query_as!(
        ArchiveRow,
        r#"
        SELECT tier, period, table_name, relative_path, row_count, byte_count, sha256,
               window_from, window_to, created_at
        FROM retention_archives
        ORDER BY period DESC, tier, table_name
        LIMIT 500
        "#,
    )
    .fetch_all(pool)
    .await
}

/// The latest daily measurement of each managed table beside the one
/// recorded closest to seven days earlier, for the growth column.
#[derive(Debug, Clone)]
pub struct MeasureRow {
    pub table_name: String,
    pub run_at: DateTime<Utc>,
    pub live_rows: i64,
    pub dead_rows: i64,
    pub total_bytes: i64,
    pub index_bytes: i64,
    pub oldest_row: Option<DateTime<Utc>>,
    pub window_days: Option<i32>,
    pub bytes_week_ago: Option<i64>,
}

pub async fn list_latest_retention_runs(pool: &PgPool) -> Result<Vec<MeasureRow>, sqlx::Error> {
    sqlx::query_as!(
        MeasureRow,
        r#"
        SELECT DISTINCT ON (r.table_name)
               r.table_name, r.run_at, r.live_rows, r.dead_rows, r.total_bytes, r.index_bytes,
               r.oldest_row, r.window_days,
               (SELECT p.total_bytes FROM retention_runs p
                 WHERE p.table_name = r.table_name AND p.tier = 'daily'
                   AND p.run_at <= r.run_at - INTERVAL '7 days'
                 ORDER BY p.run_at DESC LIMIT 1) AS bytes_week_ago
        FROM retention_runs r
        WHERE r.tier = 'daily'
        ORDER BY r.table_name, r.run_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

/// The part of the monthly health report the page reads: its ranked
/// findings. The jobs crate writes the document; sizes and the rest of it
/// stay in the row for the export.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct HealthReportDoc {
    #[serde(default)]
    pub findings: Vec<HealthFindingDoc>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct HealthFindingDoc {
    pub rank: String,
    pub check: String,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct HealthReportRow {
    pub run_at: DateTime<Utc>,
    pub report: sqlx::types::Json<HealthReportDoc>,
    pub findings_p1: i32,
    pub findings_p2: i32,
    pub findings_p3: i32,
}

pub async fn find_latest_health_report(
    pool: &PgPool,
) -> Result<Option<HealthReportRow>, sqlx::Error> {
    sqlx::query_as!(
        HealthReportRow,
        r#"
        SELECT run_at, report AS "report: sqlx::types::Json<HealthReportDoc>",
               findings_p1, findings_p2, findings_p3
        FROM retention_health_reports
        ORDER BY run_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await
}

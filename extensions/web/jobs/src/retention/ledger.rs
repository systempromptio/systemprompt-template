//! Writes to the retention ledger (`retention_runs`, `retention_archives`,
//! `retention_health_reports`) and the catalog reads that feed them.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::ManagedTable;
use super::health::HealthReport;
use crate::error::JobError;

#[derive(Debug, Clone)]
pub(crate) struct TableMeasure {
    pub table: &'static str,
    pub live_rows: i64,
    pub dead_rows: i64,
    pub total_bytes: i64,
    pub index_bytes: i64,
    pub oldest_row: Option<DateTime<Utc>>,
}

// Why: size and dead tuples come from the catalog, the age from one MIN()
// over the table's time column — the two reads a retention policy is about.
pub(crate) async fn measure(pool: &PgPool, table: &ManagedTable) -> Result<TableMeasure, JobError> {
    let stats: (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT COALESCE(s.n_live_tup, 0), COALESCE(s.n_dead_tup, 0),
                pg_total_relation_size(c.oid), pg_indexes_size(c.oid)
         FROM pg_class c
         JOIN pg_namespace n ON n.oid = c.relnamespace
         LEFT JOIN pg_stat_user_tables s ON s.relid = c.oid
         WHERE n.nspname = 'public' AND c.relname = $1",
    )
    .bind(table.name)
    .fetch_one(pool)
    .await?;
    let oldest: Option<DateTime<Utc>> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT MIN({})::timestamptz FROM {}",
        table.time_column, table.name
    )))
    .fetch_one(pool)
    .await?;
    Ok(TableMeasure {
        table: table.name,
        live_rows: stats.0,
        dead_rows: stats.1,
        total_bytes: stats.2,
        index_bytes: stats.3,
        oldest_row: oldest,
    })
}

pub(crate) async fn record_run(
    pool: &PgPool,
    tier: &str,
    measure: &TableMeasure,
    window_days: Option<i32>,
) -> Result<(), JobError> {
    sqlx::query(
        "INSERT INTO retention_runs
            (tier, table_name, live_rows, dead_rows, total_bytes, index_bytes, oldest_row, window_days)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(tier)
    .bind(measure.table)
    .bind(measure.live_rows)
    .bind(measure.dead_rows)
    .bind(measure.total_bytes)
    .bind(measure.index_bytes)
    .bind(measure.oldest_row)
    .bind(window_days)
    .execute(pool)
    .await?;
    Ok(())
}

// Why: the size recorded closest to `days` ago is the growth baseline.
pub(crate) async fn size_days_ago(
    pool: &PgPool,
    table: &str,
    days: i64,
) -> Result<Option<i64>, JobError> {
    let bytes: Option<i64> = sqlx::query_scalar(
        "SELECT total_bytes FROM retention_runs
         WHERE table_name = $1 AND tier = 'daily'
           AND run_at <= CURRENT_TIMESTAMP - make_interval(days => $2::int)
         ORDER BY run_at DESC LIMIT 1",
    )
    .bind(table)
    .bind(i32::try_from(days).unwrap_or(i32::MAX))
    .fetch_optional(pool)
    .await?;
    Ok(bytes)
}

#[derive(Debug, Clone)]
pub(crate) struct ArchiveRecord<'a> {
    pub tier: &'a str,
    pub period: &'a str,
    pub table: &'a str,
    pub relative_path: &'a str,
    pub row_count: i64,
    pub byte_count: i64,
    pub sha256: &'a str,
    pub window_from: DateTime<Utc>,
    pub window_to: DateTime<Utc>,
}

pub(crate) async fn record_archive(
    pool: &PgPool,
    record: &ArchiveRecord<'_>,
) -> Result<(), JobError> {
    sqlx::query(
        "INSERT INTO retention_archives
            (tier, period, table_name, relative_path, row_count, byte_count, sha256, window_from, window_to)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (tier, period, table_name) DO UPDATE SET
            relative_path = EXCLUDED.relative_path, row_count = EXCLUDED.row_count,
            byte_count = EXCLUDED.byte_count, sha256 = EXCLUDED.sha256,
            window_from = EXCLUDED.window_from, window_to = EXCLUDED.window_to,
            created_at = CURRENT_TIMESTAMP",
    )
    .bind(record.tier)
    .bind(record.period)
    .bind(record.table)
    .bind(record.relative_path)
    .bind(record.row_count)
    .bind(record.byte_count)
    .bind(record.sha256)
    .bind(record.window_from)
    .bind(record.window_to)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn archive_exists(
    pool: &PgPool,
    tier: &str,
    period: &str,
) -> Result<bool, JobError> {
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM retention_archives WHERE tier = $1 AND period = $2",
    )
    .bind(tier)
    .bind(period)
    .fetch_one(pool)
    .await?;
    Ok(n > 0)
}

pub(crate) async fn record_health_report(
    pool: &PgPool,
    report: &HealthReport,
    counts: (i32, i32, i32),
) -> Result<(), JobError> {
    sqlx::query(
        "INSERT INTO retention_health_reports (report, findings_p1, findings_p2, findings_p3)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(sqlx::types::Json(report))
    .bind(counts.0)
    .bind(counts.1)
    .bind(counts.2)
    .execute(pool)
    .await?;
    Ok(())
}

//! The monthly health check: the checks from `docs/ops/db-analysis/queries/
//! health/` that a job can run without operator input, as one JSON report
//! with ranked findings. P1 is a queue that is not draining or a window
//! that is not being applied; P2 is growth that will hurt within a year;
//! P3 is hygiene.

use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;

use super::all_managed;
use super::ledger::measure;
use crate::error::JobError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum Rank {
    P1,
    P2,
    P3,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HealthFinding {
    pub rank: Rank,
    pub check: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TableSize {
    pub table: &'static str,
    pub live_rows: i64,
    pub dead_rows: i64,
    pub total_bytes: i64,
    pub index_bytes: i64,
    pub oldest_row: Option<chrono::DateTime<Utc>>,
    pub window_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HealthReport {
    pub generated_at: chrono::DateTime<Utc>,
    pub tables: Vec<TableSize>,
    pub findings: Vec<HealthFinding>,
}

impl HealthReport {
    pub(crate) fn counts(&self) -> (i32, i32, i32) {
        let count = |rank| {
            i32::try_from(self.findings.iter().filter(|f| f.rank == rank).count())
                .unwrap_or(i32::MAX)
        };
        (count(Rank::P1), count(Rank::P2), count(Rank::P3))
    }
}

// Why: the outbox drains in seconds when healthy, so an hour-old pending
// row means a stalled consumer whatever the depth. Dead tuples above a fifth
// means autovacuum is behind; a
// table whose oldest row is a week past its window has a retention job that
// is not running or not enforced.
const DEAD_RATIO_WARN_PERCENT: i64 = 20;
const WINDOW_GRACE_DAYS: i64 = 7;
const OUTBOX_STALE_MINUTES: i64 = 60;
const UNUSED_INDEX_MIN_BYTES: i64 = 10 * 1024 * 1024;

pub(crate) async fn run(pool: &PgPool) -> Result<HealthReport, JobError> {
    let now = Utc::now();
    let mut tables = Vec::new();
    let mut findings = Vec::new();
    for table in all_managed() {
        let m = measure(pool, table).await?;
        if m.live_rows > 0 && m.dead_rows * 100 / m.live_rows.max(1) > DEAD_RATIO_WARN_PERCENT {
            findings.push(HealthFinding {
                rank: Rank::P3,
                check: "dead_tuples",
                detail: format!(
                    "{}: {} dead of {} live rows",
                    m.table, m.dead_rows, m.live_rows
                ),
            });
        }
        if let (Some(days), Some(oldest)) = (table.window_days, m.oldest_row) {
            let limit = now - Duration::days(i64::from(days) + WINDOW_GRACE_DAYS);
            if oldest < limit {
                findings.push(HealthFinding {
                    rank: Rank::P1,
                    check: "window_not_applied",
                    detail: format!(
                        "{}: oldest row {} is beyond its {}-day window — retention is not running or not enforced",
                        m.table,
                        oldest.date_naive(),
                        days
                    ),
                });
            }
        }
        tables.push(TableSize {
            table: m.table,
            live_rows: m.live_rows,
            dead_rows: m.dead_rows,
            total_bytes: m.total_bytes,
            index_bytes: m.index_bytes,
            oldest_row: m.oldest_row,
            window_days: table.window_days,
        });
    }
    tables.sort_by_key(|table| std::cmp::Reverse(table.total_bytes));
    check_outboxes(pool, &mut findings).await?;
    check_stuck_requests(pool, &mut findings).await?;
    check_unused_indexes(pool, &mut findings).await?;
    check_anonymous_users(pool, &mut findings).await?;
    Ok(HealthReport {
        generated_at: now,
        tables,
        findings,
    })
}

async fn check_outboxes(pool: &PgPool, findings: &mut Vec<HealthFinding>) -> Result<(), JobError> {
    let rows: Vec<(String, i64, Option<chrono::DateTime<Utc>>)> = sqlx::query_as(
        "SELECT COALESCE(consumer, '(relay)'), COUNT(*), MIN(created_at)
         FROM event_outbox WHERE processed_at IS NULL GROUP BY consumer",
    )
    .fetch_all(pool)
    .await?;
    let stale = Utc::now() - Duration::minutes(OUTBOX_STALE_MINUTES);
    for (consumer, count, oldest) in rows {
        if oldest.is_some_and(|t| t < stale) {
            findings.push(HealthFinding {
                rank: Rank::P1,
                check: "event_outbox_backlog",
                detail: format!(
                    "{count} rows pending for {consumer} since {} — the consumer is not draining",
                    oldest.map(|t| t.to_rfc3339()).unwrap_or_default()
                ),
            });
        }
    }
    Ok(())
}

async fn check_stuck_requests(
    pool: &PgPool,
    findings: &mut Vec<HealthFinding>,
) -> Result<(), JobError> {
    let stuck: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ai_requests
         WHERE status IN ('pending', 'streaming')
           AND created_at < CURRENT_TIMESTAMP - INTERVAL '1 hour'",
    )
    .fetch_one(pool)
    .await?;
    if stuck > 0 {
        findings.push(HealthFinding {
            rank: Rank::P3,
            check: "stuck_requests",
            detail: format!("{stuck} requests pending or streaming for more than an hour"),
        });
    }
    Ok(())
}

async fn check_unused_indexes(
    pool: &PgPool,
    findings: &mut Vec<HealthFinding>,
) -> Result<(), JobError> {
    let rows: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT s.relname, s.indexrelname, pg_relation_size(s.indexrelid)
         FROM pg_stat_user_indexes s JOIN pg_index i ON i.indexrelid = s.indexrelid
         WHERE s.idx_scan = 0 AND NOT i.indisunique AND NOT i.indisprimary
           AND pg_relation_size(s.indexrelid) > $1
         ORDER BY pg_relation_size(s.indexrelid) DESC LIMIT 20",
    )
    .bind(UNUSED_INDEX_MIN_BYTES)
    .fetch_all(pool)
    .await?;
    for (table, index, bytes) in rows {
        findings.push(HealthFinding {
            rank: Rank::P3,
            check: "unused_index",
            detail: format!("{table}.{index}: {bytes} bytes, never scanned since stats reset"),
        });
    }
    Ok(())
}

async fn check_anonymous_users(
    pool: &PgPool,
    findings: &mut Vec<HealthFinding>,
) -> Result<(), JobError> {
    let (anonymous, total): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*) FILTER (WHERE 'anonymous' = ANY(roles)), COUNT(*) FROM users",
    )
    .fetch_one(pool)
    .await?;
    if total > 0 && anonymous * 100 / total > 50 {
        findings.push(HealthFinding {
            rank: Rank::P2,
            check: "anonymous_users",
            detail: format!(
                "{anonymous} of {total} users are anonymous fingerprint rows — cleanup_anonymous_users is not enforced"
            ),
        });
    }
    Ok(())
}

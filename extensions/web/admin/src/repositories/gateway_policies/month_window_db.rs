//! The daily rewrite behind calendar-month quota windows.
//!
//! For every enabled policy whose spec carries a monthly window (see
//! [`super::month_window`] for why one is recognisable), three writes in
//! order:
//!
//! 1. purge any bucket already sitting on today's key that was last touched
//!    before today — a key from an earlier period that happens to coincide;
//! 2. carry yesterday's bucket into today's, adding counters, so the bucket the
//!    gateway reserves against holds month-to-date usage — skipped on the first
//!    of the month, which starts from zero;
//! 3. rewrite the policy row's `window_seconds` to today's value.
//!
//! Core caches the policy for up to sixty seconds, so requests in the first
//! minute after the rewrite may still land in yesterday's bucket and are
//! not carried. That minute is the documented cost of the interim.
//!
//! A run is idempotent per day: a window already on today's value is left
//! alone, and a subject is carried once however many rows declare it.

use std::collections::HashSet;

use chrono::{DateTime, Datelike, Duration, NaiveTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use super::month_window::{
    MONTH_WINDOW_SECONDS, align_window, is_month_window, month_window_seconds,
};
use super::rows::{PolicyWrite, list_policies, upsert_policy};
use crate::error::AdminResult;

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct MonthWindowReport {
    pub policies_rewritten: usize,
    pub windows_rewritten: usize,
    pub buckets_carried: u64,
    pub buckets_purged: u64,
}

struct BucketKey {
    window_seconds: i32,
    window_start: DateTime<Utc>,
}

async fn purge_stale(
    pool: &PgPool,
    subject_kind: &str,
    key: &BucketKey,
    today_start: DateTime<Utc>,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        r"DELETE FROM ai_quota_buckets
           WHERE subject_kind = $1 AND window_seconds = $2 AND window_start = $3
             AND updated_at < $4",
        subject_kind,
        key.window_seconds,
        key.window_start,
        today_start
    )
    .execute(pool)
    .await?;
    Ok(done.rows_affected())
}

async fn carry_forward(
    pool: &PgPool,
    subject_kind: &str,
    from: &BucketKey,
    to: &BucketKey,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query!(
        r"INSERT INTO ai_quota_buckets
              (id, subject_kind, subject_id, window_seconds, window_start,
               requests, input_tokens, output_tokens, cost_microdollars, updated_at)
          SELECT gen_random_uuid()::text, subject_kind, subject_id, $4, $5,
                 requests, input_tokens, output_tokens, cost_microdollars, CURRENT_TIMESTAMP
            FROM ai_quota_buckets
           WHERE subject_kind = $1 AND window_seconds = $2 AND window_start = $3
          ON CONFLICT (subject_kind, subject_id, window_seconds, window_start) DO UPDATE
             SET requests = ai_quota_buckets.requests + EXCLUDED.requests,
                 input_tokens = ai_quota_buckets.input_tokens + EXCLUDED.input_tokens,
                 output_tokens = ai_quota_buckets.output_tokens + EXCLUDED.output_tokens,
                 cost_microdollars = ai_quota_buckets.cost_microdollars + EXCLUDED.cost_microdollars,
                 updated_at = CURRENT_TIMESTAMP",
        subject_kind,
        from.window_seconds,
        from.window_start,
        to.window_seconds,
        to.window_start
    )
    .execute(pool)
    .await?;
    Ok(done.rows_affected())
}

pub async fn refresh_month_windows(
    pool: &PgPool,
    now: DateTime<Utc>,
) -> AdminResult<MonthWindowReport> {
    let today = now.date_naive();
    let target = month_window_seconds(today);
    // Why: discard-ok: midnight and noon exist on every date
    let today_start = today.and_time(NaiveTime::default()).and_utc();
    let yesterday_noon = (today - Duration::days(1))
        .and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap_or_default())
        .and_utc();
    let same_month = yesterday_noon.month() == now.month();

    let mut report = MonthWindowReport::default();
    let mut carried: HashSet<String> = HashSet::new();
    for row in list_policies(pool).await? {
        let mut spec = row.spec.clone();
        let mut changed = 0;
        for window in &mut spec.quota_windows {
            if !is_month_window(window.window_seconds) || window.window_seconds == target {
                continue;
            }
            let to = BucketKey {
                window_seconds: target,
                window_start: align_window(now, target),
            };
            if row.enabled && carried.insert(window.subject.clone()) {
                report.buckets_purged +=
                    purge_stale(pool, &window.subject, &to, today_start).await?;
                // Why: the sentinel is a declaration that was never live, so
                // there is no bucket behind it to carry; a live value from
                // last month is a period that has ended.
                if same_month && window.window_seconds != MONTH_WINDOW_SECONDS {
                    let from = BucketKey {
                        window_seconds: window.window_seconds,
                        window_start: align_window(yesterday_noon, window.window_seconds),
                    };
                    report.buckets_carried +=
                        carry_forward(pool, &window.subject, &from, &to).await?;
                }
            }
            window.window_seconds = target;
            changed += 1;
        }
        if changed > 0 {
            upsert_policy(
                pool,
                &PolicyWrite {
                    name: &row.name,
                    spec: &spec,
                    enabled: row.enabled,
                    priority: row.priority,
                },
            )
            .await?;
            report.policies_rewritten += 1;
            report.windows_rewritten += changed;
        }
    }
    Ok(report)
}
